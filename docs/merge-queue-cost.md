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

The after period runs from the landing group's creation, 2026-10-18T00:32:14Z, to the last
job completion among these eight, 2026-10-18T22:28:02Z, when 39170956731's
`cargo test (windows-latest)` completed. The next group, 39175877957, was created at
2026-10-18T23:17:20Z, after the eighth group's run last updated at 22:28:03Z, so the eight
are every group in the period.

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
- **Search bound:** overlapping runs were sought among all 1,447 `ci.yml` runs created
  from 2026-09-17T00:00Z through 2026-10-18T23:59:59Z, of every event and conclusion. The
  bound starts 30 days before the first group, because the forge re-runs a run for up to 30
  days and a re-run stretches its span: the push
  [38827147761](https://github.com/agent-grounds/grund/actions/runs/38827147761), created
  2026-10-16T00:02:27Z and re-run seven times on 2026-10-17, spans 43.8 hours. Of the runs
  created before 2026-10-16, one was still updating when the first group was created, and
  it is a merge-queue group rather than a `pull_request` or `push` run.
- **Re-runs:** every attempt's jobs are read, from
  `gh api repos/agent-grounds/grund/actions/runs/<id>/attempts/<n>/jobs --paginate`. A
  re-run of selected jobs lists every job again, but a job it did not re-run keeps the
  earlier attempt's timestamps. That is the earlier execution, not a new sample, so it is
  counted once, and its completion still resolves the needs of a job that was re-run. A
  re-run job becomes eligible no earlier than its attempt was requested, and one with no
  needs when it was requested. A run re-run after it finished spans the time it sat idle,
  so it overlaps a group it was idle through without a job that qualifies there. Every
  group ran once. Three overlapping runs were re-run.
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

Every value in these rows was derived again from the saved run and job records, once with
#798's search for overlapping runs, those created from 2026-10-17T13:00Z, and once with the
search bound above. Every elapsed time, job count, last suite leg and start delay
reproduces. Two kinds of value do not, and the rows above stay as #798 published them:

- **39057461193:** with #798's own search it had 2 overlapping runs, not 1, and their max
  delay was 4 s, not 3. Their median stays 3. The run left out is the pull request run
  [39057106223](https://github.com/agent-grounds/grund/actions/runs/39057106223), created
  at 18:12:51Z. Its second attempt was requested at 18:35:22Z, inside the group's span, and
  its 19 jobs started 2–4 s after they became eligible. A job list read without naming the
  attempt returns only its fourth attempt, from 19:35Z, which would explain the omission.
- **The wider search** adds the push
  [38827147761](https://github.com/agent-grounds/grund/actions/runs/38827147761) as an
  overlapping run of five groups. It was created 2026-10-16T00:02:27Z and re-run seven
  times on 2026-10-17 between 14:38Z and 19:51Z. So 39057461193, 39054668908, 39050025966,
  39046619762 and 39039527929 have 3, 3, 6, 9 and 2 overlapping runs, where #798 counted
  1, 2, 5, 8 and 1. The push adds one qualifying job, to 39057461193 alone: its fourth
  attempt's `rust (ubuntu-latest)`, eligible at 18:37:59Z, started after 4 s. So that
  group's max delay stays 4 and its median 3.

Neither correction moves a summary. With 4 in place of 3, the eight groups' overlapping
runs' max delays are 289, 55, 587, 4, 232, 1145, 784 and 3, still 260.5 (3–1145). Under the
wider search the delay columns pool 12, 21, 36, 21, 19, 59, 69 and 17 qualifying jobs, in
the order of the rows.

## After

The eight groups after #798, derived the same way. Every one ran a `ci.yml` that runs the
bench jobs on a group, so it started 19 jobs: the 10 the before groups started, the build,
seven shards and the record. Each succeeded on its first attempt. The first five ran the
landing group's `ci.yml`. The last three ran a revision that changed a step. Every run in
this report, the groups and the runs beside them, declares the same jobs and needs.

| Group | Elapsed (s) | Jobs started | Last suite leg, its duration | Max start delay (s) | Median start delay (s) | Overlapping runs | Their max delay (s) | Their median delay (s) |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| [39170956731](https://github.com/agent-grounds/grund/actions/runs/39170956731) | 1145 | 19 | node acceptance (windows-latest), 1135 s | 3 | 3 | 1 | 3 | 2 |
| [39158389971](https://github.com/agent-grounds/grund/actions/runs/39158389971) | 1310 | 19 | node acceptance (windows-latest), 1303 s | 3 | 3 | 1 | 7 | 3.0 |
| [39147247717](https://github.com/agent-grounds/grund/actions/runs/39147247717) | 1200 | 19 | rust (windows-latest), 1192 s | 4 | 2 | 2 | 3 | 3.0 |
| [39146359915](https://github.com/agent-grounds/grund/actions/runs/39146359915) | 1342 | 19 | node acceptance (windows-latest), 1335 s | 4 | 3 | 3 | 2 | 2.0 |
| [39142426332](https://github.com/agent-grounds/grund/actions/runs/39142426332) | 1172 | 19 | rust (windows-latest), 1164 s | 5 | 3 | 1 | 3 | 2 |
| [39130387455](https://github.com/agent-grounds/grund/actions/runs/39130387455) | 1406 | 19 | node acceptance (windows-latest), 1399 s | 3 | 2 | 0 | N/A (0 qualifying jobs) | N/A (0 qualifying jobs) |
| [39102206632](https://github.com/agent-grounds/grund/actions/runs/39102206632) | 1033 | 19 | python (windows-latest), 1024 s | 178 | 3 | 1 | 2 | 2.0 |
| [39092806613](https://github.com/agent-grounds/grund/actions/runs/39092806613) | 1332 | 19 | node acceptance (windows-latest), 1324 s | 3 | 2 | 0 | N/A (0 qualifying jobs) | N/A (0 qualifying jobs) |

The delay columns pool 9, 16, 16, 4, 9, 0, 10 and 0 qualifying jobs, in the order of the
rows. 39130387455 and 39092806613 had no overlapping run. The counts of 39146359915 and
39147247717 include the push
[39143933517](https://github.com/agent-grounds/grund/actions/runs/39143933517). It finished
at 16:39:02Z and was re-run at 18:44:58Z, so it sat idle through both groups and adds no
delay to either.

| Group | Most jobs running at once | Last shard started after the build completed (s) | cargo test (ubuntu-latest) completed before cargo test (windows-latest) by (s) |
|---|---:|---:|---:|
| [39170956731](https://github.com/agent-grounds/grund/actions/runs/39170956731) | 14 | 3 | 613 |
| [39158389971](https://github.com/agent-grounds/grund/actions/runs/39158389971) | 13 | 3 | 502 |
| [39147247717](https://github.com/agent-grounds/grund/actions/runs/39147247717) | 14 | 2 | 534 |
| [39146359915](https://github.com/agent-grounds/grund/actions/runs/39146359915) | 14 | 3 | 852 |
| [39142426332](https://github.com/agent-grounds/grund/actions/runs/39142426332) | 13 | 2 | 525 |
| [39130387455](https://github.com/agent-grounds/grund/actions/runs/39130387455) | 13 | 3 | 752 |
| [39102206632](https://github.com/agent-grounds/grund/actions/runs/39102206632) | 10 | 178 | 418 |
| [39092806613](https://github.com/agent-grounds/grund/actions/runs/39092806613) | 13 | 3 | 737 |

Seven of the eight groups had 13 or 14 jobs running at once. The plan projected about 12,
and the landing group had 11. The before groups peaked at 5 to 8. 39102206632 peaked at 10
because five of its shards waited for runners: they started 78–178 s after the build
completed, and that wait is the group's max start delay. In every other group all seven
shards started within 3 s of the build. The bench chain took 407–606 s from the build's
start to the record's completion. In every group `cargo test (ubuntu-latest)` completed
before `cargo test (windows-latest)`, by 418–852 s. The Windows aggregate was each group's
last job, so a Windows leg, not the bench chain, set every group's elapsed time.

## Comparison

The median and range of each cohort's per-group figures, in seconds, with the landing
group as #798 reported it.

| Groups | Elapsed median (range), s | Max start delay median (range), s | Overlapping runs' max delay median (range), s |
|---|---|---|---|
| 8 before #798 | 1258.0 (1107–1859) | 7.0 (3–864) | 260.5 (3–1145) |
| [39083120016](https://github.com/agent-grounds/grund/actions/runs/39083120016), the group #798 landed in | 1356 | 144 | 25 |
| 8 after #798 | 1255.0 (1033–1406) | 3.5 (3–178) | 3.0 (2–7), 2 missing |

The after overlap cell leaves out 39130387455 and 39092806613, which had no overlapping
run. The landing row reproduces from the saved records, and so do its other figures in
#798's body: 19 jobs started, at most 11 running at once, four shards starting 130–144 s
after the build and three within 49 s, 604 s for the bench chain, 746 s between the two
aggregates, and 1349 s for its last Windows leg.

## Bench failures in the period

None. Nine merge-queue groups were created in the period: the landing group, at its start,
and the eight after groups. The forge lists no other `merge_group` run of `ci.yml` created
in it. Each group ran once, and each of its nine bench jobs, `cargo bench (build)`, the
seven shards and `cargo bench (record)`, concluded `success` on that attempt: 81 bench jobs
in all. A green group cannot hide a red shard, because
`tests/integration/test_merge_queue_runs.py` refuses `continue-on-error` on a group's shard
job and on its ceiling step. So no failure needed a cause: there was no ceiling crossing
the pull request could not see, no same-binary jitter, and no runner or forge failure.

## Assessment

The groups after #798 are not slower. By the criterion #800 set, an after median above the
before median on any of the three measures would mean slower, and none is above it.

- **Elapsed:** the median is 1255.0 s after against 1258.0 s before, 3.0 s shorter. The
  range is 1033–1406 s against 1107–1859 s.
- **The groups' start delay:** the median of each group's max start delay is 3.5 s after
  against 7.0 s before, 3.5 s shorter. The range is 3–178 s against 3–864 s.
- **The overlapping runs' start delay:** the median of each group's overlapping runs' max
  delay is 3.0 s after against 260.5 s before, 257.5 s shorter. The range is 2–7 s against
  3–1145 s. Two after groups had no overlapping run, so the after median is over six.

The corrections to the before rows leave all three before medians where #798 put them, so
the verdict holds under either search. What did grow is what
[§AR-ci.12](architecture/AR-ci.md#12-a-merge-queue-lands-every-pull-request) said would
grow, concurrency. A group now starts 19 jobs where it started 10. Seven of the eight had 13 or
14 running at once, against 5 to 8 before and more than the plan's projection of about 12.

These figures show what happened in sixteen groups, not why.

- Each cohort is eight groups.
- Every delay includes runner provisioning, and the forge keeps no history of runner
  capacity.
- The after groups ran beside fewer runs: 9 overlapping runs over the eight groups, two of
  them the idle push 39143933517, where the before groups had 30 under the wider search.
  So the overlap comparison says little about contention under the load the before groups
  saw.
- The one after group that waited for runners, 39102206632, shows the concurrency cost can
  reach a group: five of its shards waited up to 178 s. Its `cargo test (ubuntu-latest)`
  still completed 418 s before `cargo test (windows-latest)`.

No optimization issue is needed.
