# Pending-interrupt harness regression contract (#875)

The behavior is [§FS-distribution.3.3.4](../../../docs/functional-spec/FS-distribution.md#334-silent-synchronous-calls):
deliver an interrupt pending during native work at the call's return, without
promising mid-call cancellation. The next call against an independent root still
succeeds under [§FS-distribution.3.3.3](../../../docs/functional-spec/FS-distribution.md#333-per-call-scope-and-path-encoding).
The evidence mechanics are defined once at
[§AR-bindings.6.2.1](../../../docs/architecture/AR-bindings.md#621-pending-interrupt-acceptance-evidence).

`test_interrupt_evidence.InterruptEvidenceTests` runs the live
`test_boundary.BoundaryTests.test_pending_interrupt_is_delivered_when_operation_returns`
through `interrupt_evidence_probe.py`; it does not copy that test's sender or
replace its native call. With this checkout installed from source in the chosen
Python interpreter, run all three regressions from the repository root:

```bash
PYTHONPATH=tests/bindings:tests/bindings/python python -m unittest test_interrupt_evidence -v
```

Before any harness repair, all three fail on Linux with Python 3.11.5 at
7c566b062771ff7d251fc78bc319aa91e310d840:

- `test_live_acceptance_with_ignoring_parent`: child inherits SIG_IGN; live
  acceptance exits 1 with `AssertionError: pending interrupt was swallowed`.
- `test_live_acceptance_with_blocked_parent`: child inherits blocked SIGINT;
  live acceptance exits 1 with `AssertionError: pending interrupt was swallowed`.
- `test_held_sender_reports_unavailable_evidence`: helper is held behind a
  never-released pipe before executing any command. The event record shows
  `exception_before_cleanup: "AssertionError('pending interrupt was swallowed')"`
  with `helper_alive: true` and `gate_released: false`. The outer regression fails
  with `'pending interrupt was swallowed' unexpectedly found`, because no signal
  was sent. Cleanup kills the helper only after recording the failure decision.

The inherited-state cases must pass after repair. The held-helper case must
instead observe a bounded harness failure containing `interrupt evidence
unavailable`, without a swallowed-interrupt verdict. It requires an actual
`helper-held` event, so losing the instrumentation match cannot make it pass.
The probe holds all subprocess helpers launched by the live `-c` child, without
matching the old sender's source, and records readiness through a bounded pipe
acknowledgment before declaring the helper held. If the child/helper transport changes,
adapt this test instrumentation and retain the control's meaning.

These are negative scheduling evidence and inherited-state regressions. They do
not yet prove successful synchronized delivery during native work. Implementation
must add that positive evidence with its chosen private protocol, demonstrate
KeyboardInterrupt at the intended native call's return, and retain the independent
follow-up call. A signal in setup or later Python work cannot satisfy it. The
existing Linux native-read validation control already passes and must stay
passing; it is neither a pre-fix failing regression nor a portable replacement
for the POSIX acceptance. The historical failure's exact trigger remains unknown.

The specification commit leaves the live harness unchanged and these three
regressions red, with no temporary skip markers: this checkout has no installed
pre-commit hook requiring skips to commit them. Implementation repairs the live
harness, adapts the scheduling instrumentation if necessary, and runs the complete
gate. No production boundary or public testing API change is called for.
