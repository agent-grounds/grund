# DF-release-candidates-follow-main: a release helper follows a main the merge queue moves, rebuilding and re-running its candidates

**Date:** 2026-10-23

## status: Accepted

## 1. Context

Two contracts met in agent-grounds/grund#983. A release helper moves `main` only to commits its candidate runs passed, and "`main` having moved since the helper started still refuses the release" ([§FS-distribution.4.4.1](../../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). Every pull request lands through the merge queue ([§AR-ci.12](../../architecture/AR-ci.md#12-a-merge-queue-lands-every-pull-request)), so `main` moves whenever the queue lands one. Each rule held up alone; together they gave a release a window only when nothing merged for the hour of candidate CI. On 2026-10-23 `main` took 19 merges, 45 minutes apart at the median, and run 39962684811 of `release-minor.yml` was refused at its end, after three full CI runs had passed, because the queue landed #975 three minutes before they finished. The refusal sentence was written before the queue; the guarantee beside it, that nothing untested reaches `main`, was the one worth keeping ([§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

## 2. Decision

### 2.1 Follow the move, in bounded rounds

A helper not pinned to a tip gives a round up as soon as `main` moves - cancels its runs, deletes its branches - waits for `main` to hold still for three minutes, rebuilds both candidates on the new tip by the whole preparation, and runs all three runs again ([§FS-distribution.4.4.1.1](../../functional-spec/FS-distribution.md#4411-a-moved-main-is-followed-not-refused)). It runs at most six rounds in a 300-minute job, starts one only with 90 minutes left, and gives its watch a deadline that leaves room to publish ([§FS-distribution.4.4.1.2](../../functional-spec/FS-distribution.md#4412-six-rounds-within-five-hours)). `main` only fast-forwards, to commits the last round's runs passed, and a push the queue beat is a move like any other ([§FS-distribution.4.4.1.3](../../functional-spec/FS-distribution.md#4413-the-push-to-main-reads-main-back)).

### 2.2 Rebuild, never replay

A round builds on the new tip from the start rather than rebasing the old candidates onto it, so a merged pull request that adds a ramp, a notice or a version change is met by the same preparation that would have met it at dispatch, and its section lists what merged. Rebasing without running the candidates again would push commits no run passed.

### 2.3 A pinned run stays frozen

A run given `expected_main_sha` was dispatched to release that tip and no other, so it follows no move: it cancels its runs, deletes its branches and names the new tip to pin ([§FS-distribution.4.4.2](../../functional-spec/FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)). The only change for it is that the refusal comes when the move is seen, not an hour later.

### 2.4 Only a failure leaves branches

A run that did not pass keeps its round's branches for a person; an abandoned round, exhaustion and a pinned refusal delete theirs, because nothing on them is evidence ([§FS-distribution.4.4.1.4](../../functional-spec/FS-distribution.md#4414-only-a-failure-leaves-branches)).

## 3. Rejected alternatives

Holding the merge queue while the candidates run blocks every contributor for an hour, and nothing reliable releases the hold when a job dies. Sending the release commit through the queue would test it with whatever is queued ahead of it, but the release and its `-dev` advance are two commits validated together, and following them through the queue's changing group identities is a harder contract than rebuilding. Replaying the old candidates onto the new tip fails on a conflict a rebuild never meets, and carries over a release section that no longer lists what merged.

## 4. Consequences

A release dispatched at any time keeps up with the queue on its own, instead of needing a quiet hour picked by hand, and may now include pull requests merged after it was dispatched. Finite rounds are not a promise: a queue that never holds still for an hour in five exhausts the helper, which publishes nothing and says so. In the worst case the helper holds runners for 300 minutes rather than one hour, though an abandoned round stops within a poll. The publisher is dispatched with `expected_release_sha` ([§FS-distribution.4.3](../../functional-spec/FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)) on the branch its runs passed. No command, configuration or finding of `grund` changes, so the decision carries no release note.
