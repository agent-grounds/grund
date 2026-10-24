# RFC-0012-jittered-backoff: Full jitter for the retry schedule

- Feature Name: `jittered_backoff`
- Start Date: 2026-05-18
- RFC PR: [skerry/rfcs#12](https://forge.example/skerry/rfcs/pull/12)
- Skerry Issue: [skerry/skerry#58](https://forge.example/skerry/skerry/issues/58)

## summary: Summary
[summary]: #summary-summary

Let a `RetryPolicy` scale each delay by a random factor in `[0, 1)`, so that
clients that failed together stop retrying together. This answers the question
[§RFC-0007-retry-policy.unresolved-questions.jitter](0007-retry-policy.md#unresolved-questionsjitter-randomized-delays)
left open.

## motivation: Motivation
[motivation]: #motivation-motivation

[§RFC-0007-retry-policy.drawbacks](0007-retry-policy.md#drawbacks-drawbacks)
names the cost of its own schedule: every client follows the same delays, so a
recovering dependency meets all of its clients at once. Spreading each delay
over its whole range breaks that synchrony without changing the cap.

## guide-level-explanation: Guide-level explanation
[guide-level-explanation]: #guide-level-explanation-guide-level-explanation

Add jitter to a policy by naming where its random numbers come from:

```rust
let policy = RetryPolicy::exponential(Duration::from_millis(100), Duration::from_secs(5), 6)
    .with_jitter(fastrand::f64);
```

The fourth retry, whose capped delay is 800 ms, now waits anywhere from 0 to
800 ms. A test passes a fixed source, such as `|| 0.5`, and gets half of each
delay every time.

## reference-level-explanation: Reference-level explanation
[reference-level-explanation]: #reference-level-explanation-reference-level-explanation

### reference-level-explanation.jitter: The jittered delay

With jitter, `delay(n)` is `d × r`, where `d` is the delay
[§RFC-0007-retry-policy.reference-level-explanation.schedule](0007-retry-policy.md#reference-level-explanationschedule-the-delay-schedule)
gives for `n` and `r` is one value drawn from the source. The retry limit is
unchanged.

### reference-level-explanation.source: The random source

`with_jitter(source)` takes a `fn() -> f64` that returns values in `[0, 1)`
and calls it once per `delay` that returns `Some`.

## drawbacks: Drawbacks
[drawbacks]: #drawbacks-drawbacks

A jittered policy is no longer a pure function of the retry number, so a log
of delays cannot be checked against the schedule alone.

## rationale-and-alternatives: Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives-rationale-and-alternatives

- Equal jitter keeps half of each delay fixed and randomizes the other half.
  It spreads clients less, for a guaranteed minimum wait.
- Decorrelated jitter derives each delay from the previous one. It needs state
  between calls, which `delay` does not keep.

## prior-art: Prior art
[prior-art]: #prior-art-prior-art

Ethernet's binary exponential backoff picks a random slot count below a
doubling bound, which is full jitter over a slotted schedule.

## unresolved-questions: Unresolved questions
[unresolved-questions]: #unresolved-questions-unresolved-questions

### unresolved-questions.seeding: A seedable default source

Whether skerry should ship a default source that a test can seed, rather than
asking every caller to name one. To be settled before stabilization.

## future-possibilities: Future possibilities
[future-possibilities]: #future-possibilities-future-possibilities

Decorrelated jitter, once `RetryPolicy` keeps state between retries.
