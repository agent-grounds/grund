# LC-1 charge controller: software requirements

The requirements on the charge-control software, each derived from a system
requirement in [system.md](system.md) and written in the same form.

## SWR-001: Open the charge switch above the temperature limit

When a valid temperature reading exceeds `45 °C` (§PAR-001), the charge-control
task shall open the charge switch.

### verification: Test

### rationale: Rationale

The charge switch is the only path for charge current, so opening it stops
charging.

### trace: Traces to

[§SYS-002](system.md#sys-002-stop-charging-above-the-temperature-limit)

## SWR-002: Run the charge-control task every period

The charge-control task shall run every `100 ms` (§PAR-003).

### verification: Analysis

### rationale: Rationale

One period from reading to switch is what lets the charger meet its
`500 ms` (§PAR-002) reaction time. Only an analysis of the scheduler can show the
worst case; a test samples it.

### trace: Traces to

[§SYS-002](system.md#sys-002-stop-charging-above-the-temperature-limit)

## SWR-003: Report an out-of-range thermistor as invalid

If the thermistor reading lies outside its valid range, then the temperature
monitor shall report the temperature as invalid.

### verification: Test

### rationale: Rationale

An open thermistor reads near full scale and a shorted one near zero. Neither is
a temperature.

### trace: Traces to

[§SYS-004](system.md#sys-004-stop-when-the-temperature-cannot-be-measured)

## SWR-004: Latch the fault on an invalid temperature

When the temperature monitor reports an invalid temperature, the charge-control
task shall open the charge switch and latch the fault state.

### verification: Test

### rationale: Rationale

The fault state is what keeps the switch open afterwards.

### trace: Traces to

[§SYS-004](system.md#sys-004-stop-when-the-temperature-cannot-be-measured)

## SWR-005: Keep the switch open while the fault is latched

While the fault state is latched, the charge-control task shall keep the charge
switch open.

### verification: Test

### rationale: Rationale

Only a power cycle clears the latch, so service sees the unit as it failed.

### trace: Traces to

[§SYS-003](system.md#sys-003-stay-stopped-after-a-fault)

## SWR-006: Log the cause of every stop

The charge-control task shall write an event-log entry naming the cause each
time it opens the charge switch.

### verification: Test

### rationale: Rationale

The event log is the only record a returned unit carries.

### trace: Traces to

[§SYS-001](system.md#sys-001-record-why-charging-stopped)

## SWR-007: Blink the LED while the fault is latched

Where the status LED is fitted, the indicator task shall toggle the LED once per
control period while the fault state is latched.

### verification: Test

### rationale: Rationale

At one toggle per period the LED blinks at a rate no one mistakes for steady.

### trace: Traces to

[§SYS-005](system.md#sys-005-show-the-fault-on-the-status-led)
