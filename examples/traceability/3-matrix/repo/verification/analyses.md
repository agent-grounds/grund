# LC-1 charge controller: verification by analysis

The records of what was verified by analysis rather than by test. Each names the
software requirement it verifies and its verdict.

## ANA-001: Over-temperature reaction time

The worst case is a reading that crosses the limit just after the task ran. The
next run, one period of `100 ms` (§PAR-003) later, reads it and opens the switch;
the run takes under 1 ms on the target. The reaction time is therefore at most
101 ms, inside the budget of `500 ms` (§PAR-002).

### verdict: Pass

### trace: Traces to

[§SWR-002](../requirements/software.md#swr-002-run-the-charge-control-task-every-period)
