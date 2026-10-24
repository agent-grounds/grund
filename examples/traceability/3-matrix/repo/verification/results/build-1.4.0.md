# Verification results: build 1.4.0

- Software: LC-1 charge-control software, build 1.4.0
- Date: 2026-10-20
- Tester: Mira Castellan
- Environment: host unit tests built with `cc -std=c99 -DCONFIG_STATUS_LED`
- Pass criterion: every assertion of the test case holds

| Test case | Result |
|---|---|
| [§TC-001](../../tests/test_charge_control.c) | Pass |
| [§TC-002](../../tests/test_charge_control.c) | Pass |
| [§TC-003](../../tests/test_charge_control.c) | Pass |
| [§TC-004](../../tests/test_charge_control.c) | Pass |
| [§TC-005](../../tests/test_charge_control.c) | Pass |
| [§TC-006](../../tests/test_temp_monitor.c) | Pass |

| Analysis | Verdict |
|---|---|
| [§ANA-001](../analyses.md#ana-001-over-temperature-reaction-time) | Pass |
