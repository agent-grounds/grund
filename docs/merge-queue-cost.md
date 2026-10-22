# Merge-queue cost report

This archival report measures what it costs to hold every merge-queue group to the
instruction ceilings ([§AR-ci.12](architecture/AR-ci.md#12-a-merge-queue-lands-every-pull-request)).
That point says no gate's time grows, and that the cost is concurrency: a group's
benchmark jobs take runners from the organization's shared limit on concurrent jobs, so
another run may wait to start. [#798](https://github.com/agent-grounds/grund/pull/798)
made every group run the benchmark jobs, and the plan approved on
[#792](https://github.com/agent-grounds/grund/issues/792) said that cost would be measured
by comparing eight groups from before the change with eight from after it. This report is that
comparison, for [#800](https://github.com/agent-grounds/grund/issues/800). Every group in
it is a finished run read from the forge. The tables record those sixteen groups. They
do not monitor the queue as it runs today.

## Cohorts

- **Before:** the eight groups in #798's body, created 2026-10-17 from 14:09:14Z to
  21:42:18Z and read for it on 2026-10-18. They are kept as #798 published them. Two
  successful groups created after the last of them and before the landing group,
  39077825506 and 39079573671, are not added, so the reference is the one the plan was
  approved against.
- **Landing:** [39083120016](https://github.com/agent-grounds/grund/actions/runs/39083120016),
  the group #798 landed in, created 2026-10-18T00:32:14Z, belongs to neither cohort. Its
  figures stand beside the before rows as a single sample.
- **After:** the first eight successful `merge_group` runs of `ci.yml` created strictly
  after the landing group, in order of creation. The forge was read at
  2026-10-22T11:42:26Z without a conclusion filter. Every merge-queue group created in
  that stretch succeeded on its first attempt, so no failed, cancelled or unfinished group
  falls between them.

| Group | Created | Head |
|---|---|---|
| [39092806613](https://github.com/agent-grounds/grund/actions/runs/39092806613) | 2026-10-18T03:12:35Z | `c8f7e7b69e` |
| [39102206632](https://github.com/agent-grounds/grund/actions/runs/39102206632) | 2026-10-18T05:55:14Z | `55127fb1f8` |
| [39130387455](https://github.com/agent-grounds/grund/actions/runs/39130387455) | 2026-10-18T13:10:34Z | `b3dee6218b` |
| [39142426332](https://github.com/agent-grounds/grund/actions/runs/39142426332) | 2026-10-18T15:52:14Z | `3b10664748` |
| [39146359915](https://github.com/agent-grounds/grund/actions/runs/39146359915) | 2026-10-18T16:44:11Z | `81585799a0` |
| [39147247717](https://github.com/agent-grounds/grund/actions/runs/39147247717) | 2026-10-18T16:55:55Z | `5a536c3387` |
| [39158389971](https://github.com/agent-grounds/grund/actions/runs/39158389971) | 2026-10-18T19:21:24Z | `d76f5248fc` |
| [39170956731](https://github.com/agent-grounds/grund/actions/runs/39170956731) | 2026-10-18T22:08:57Z | `b9c3c8a552` |

The after period runs from the landing group's creation to the last job completion among
these eight. The next group, 39175877957, was created at 2026-10-18T23:17:20Z, after the
eighth group's run last updated at 22:28:03Z, so the eight are every group in the period.

## Method

The derivation is #798's, so that the two cohorts compare. Every figure is a difference
of the forge's `created_at`, `started_at` and `completed_at` values: a group's from
`gh api repos/agent-grounds/grund/actions/runs/<id>/jobs --paginate`, and those of the
runs beside it from
`gh api 'repos/agent-grounds/grund/actions/workflows/ci.yml/runs?created=<from>..<to>'`.

- **Elapsed:** run creation to the last job's completion.
- **Jobs started:** the jobs the run started. A skipped job is not a sample of anything,
  even where the forge gives it timestamps.
- **Last suite leg:** the language or pre-commit suite job that completed last, with its
  own duration, its completion less its start. The two aggregates and the commit-message
  job are not suite legs.
- **Start delay:** when a job started minus when it became eligible. A job with no needs
  becomes eligible at run creation. Any other job becomes eligible when the last of its
  needs completes, as the `ci.yml` at the run's head declares them: each
  `cargo bench (shard N)` needs `cargo bench (build)`, `cargo bench (record)` needs the
  shards, `cargo test (ubuntu-latest)` needs the four Ubuntu suites and the three bench
  jobs, and `cargo test (windows-latest)` needs the three Windows suites
  ([§AR-ci.13.2](architecture/AR-ci.md#132-the-required-check-is-an-aggregate)). A need
  the run skipped completes when the forge says it did. Every before group skipped the
  bench jobs.
- **Overlapping run:** a `pull_request` or `push` run of `ci.yml` whose span, from
  creation to last update, meets the group's span, from creation to the last job's
  completion, endpoints included. A run created before the group counts while it is still
  running. Its delays are those of its started jobs that became eligible inside the
  group's span, including a job that started after the group ended. A group's two delay
  columns pool those jobs over all its overlapping runs. A run with no such job still
  counts as an overlapping run, and where no job qualifies the columns read N/A, never 0.
- **Median:** for an even count, the mean of the two middle values.
- **Summary:** the median and range of the elapsed times, of each group's maximum start
  delay, and of each group's overlapping runs' maximum delay. A group with no overlap
  maximum is counted as missing rather than filled in.
- **Bench chain:** the most jobs a group had running at once, where a job runs from its
  start up to but not including its completion. Also how long after the build completed
  the last shard started, and how far apart the two aggregates completed.
- **Bench failures:** every group of the after period that failed in a bench job on any
  attempt, with its cause. The cause is one of three: a ceiling crossing the pull request
  could not see, which means the gate worked; same-binary jitter on a scenario at its
  ceiling ([§AR-benchmarks.7.3](architecture/AR-benchmarks.md#73-the-same-binary-bound));
  or a runner or forge failure. A cause the logs do not support is left unresolved.
- **Limits:** delays include runner provisioning, because the forge keeps no history of
  runner capacity. The comparison therefore shows what happened in sixteen groups, not
  why. #798 searched for overlapping runs from 2026-10-17T13:00Z, so it could miss a run
  created earlier and still running. The before rows are kept as published. Where a wider
  search changes one, the corrected value stands beside it with its source.

## Before

The eight groups before #798, as its body published them. Every one ran a `ci.yml` from
before the change, so its bench jobs were skipped and it started 10 jobs.

| Group | Elapsed (s) | Jobs started | Last suite leg, its duration | Max start delay (s) | Median start delay (s) | Overlapping runs | Their max delay (s) | Their median delay (s) |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| [39072064618](https://github.com/agent-grounds/grund/actions/runs/39072064618) | 1475 | 10 | rust (windows-latest), 1466 s | 10 | 3.0 | 2 | 289 | 95.5 |
| [39067732329](https://github.com/agent-grounds/grund/actions/runs/39067732329) | 1218 | 10 | node acceptance (windows-latest), 1211 s | 3 | 2.5 | 2 | 55 | 2 |
| [39066449072](https://github.com/agent-grounds/grund/actions/runs/39066449072) | 1174 | 10 | rust (windows-latest), 1167 s | 235 | 3.5 | 3 | 587 | 275.0 |
| [39057461193](https://github.com/agent-grounds/grund/actions/runs/39057461193) | 1298 | 10 | rust (windows-latest), 1292 s | 3 | 2.0 | 1 | 3 | 3 |
| [39054668908](https://github.com/agent-grounds/grund/actions/runs/39054668908) | 1191 | 10 | rust (windows-latest), 1185 s | 3 | 3.0 | 2 | 232 | 3 |
| [39050025966](https://github.com/agent-grounds/grund/actions/runs/39050025966) | 1525 | 10 | rust (windows-latest), 1214 s | 306 | 3.0 | 5 | 1145 | 119 |
| [39046619762](https://github.com/agent-grounds/grund/actions/runs/39046619762) | 1859 | 10 | rust (windows-latest), 1192 s | 864 | 605.5 | 8 | 784 | 227 |
| [39039527929](https://github.com/agent-grounds/grund/actions/runs/39039527929) | 1107 | 10 | python (windows-latest), 1100 s | 4 | 3.0 | 1 | 3 | 3 |

## Comparison

The median and range of each cohort's per-group figures, in seconds, with the landing
group as #798 reported it.

| Groups | Elapsed median (range), s | Max start delay median (range), s | Overlapping runs' max delay median (range), s |
|---|---|---|---|
| 8 before #798 | 1258.0 (1107–1859) | 7.0 (3–864) | 260.5 (3–1145) |
| [39083120016](https://github.com/agent-grounds/grund/actions/runs/39083120016), the group #798 landed in | 1356 | 144 | 25 |
