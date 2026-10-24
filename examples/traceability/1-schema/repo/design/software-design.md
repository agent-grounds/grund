# LC-1 charge controller: software design

The low-level design of the charge-control software: what each unit in `src/`
does, precisely enough to code and to test against. Each design element traces
to the software requirements in
[software.md](../requirements/software.md) it realizes.

## DES-001: Temperature monitor

`temp_monitor_read()` converts one thermistor ADC count to whole degrees
Celsius. A count outside the valid range below is not a temperature, and the
function returns `TEMP_INVALID` instead. Inside the range, a linear fit of the
thermistor curve is accurate to one degree over the charging range.

### 1. Valid ADC range <!-- grund:value -->
#### 1.1. 100
#### 1.2. 3900

### trace: Traces to

[§SWR-003](../requirements/software.md#swr-003-report-an-out-of-range-thermistor-as-invalid)

## DES-002: Charge-control state machine

`charge_control_step()` runs once per control period, on the temperature the
monitor read. The controller starts in `CHARGING`.

### 1. Transitions

| From | Condition | To | Event-log cause |
|---|---|---|---|
| `CHARGING` | temperature above `CHARGE_TEMP_MAX_C` | `STOPPED` | `CAUSE_OVER_TEMP` |
| `CHARGING`, `STOPPED` | temperature is `TEMP_INVALID` | `FAULT` | `CAUSE_SENSOR_FAULT` |

Each transition opens the charge switch and writes its cause to the event log.
`STOPPED` lasts until the supply is reconnected, which restarts the controller.

### 2. The fault latch

In `FAULT` the step opens the charge switch on every call and reads nothing else.
Only a restart leaves `FAULT`.

### 3. Period

A timer interrupt calls the step every `100 ms` (§PAR-003). The step never
blocks.

### trace: Traces to

[§SWR-001](../requirements/software.md#swr-001-open-the-charge-switch-above-the-temperature-limit),
[§SWR-002](../requirements/software.md#swr-002-run-the-charge-control-task-every-period),
[§SWR-004](../requirements/software.md#swr-004-latch-the-fault-on-an-invalid-temperature),
[§SWR-005](../requirements/software.md#swr-005-keep-the-switch-open-while-the-fault-is-latched),
[§SWR-006](../requirements/software.md#swr-006-log-the-cause-of-every-stop)

## DES-003: Status indicator

`indicator_step()` runs after the charge-control step. In `FAULT` it inverts the
LED output; in any other state it holds the LED off. It is compiled only when
`CONFIG_STATUS_LED` is defined, which a variant with the LED fitted sets.

### trace: Traces to

[§SWR-007](../requirements/software.md#swr-007-blink-the-led-while-the-fault-is-latched)
