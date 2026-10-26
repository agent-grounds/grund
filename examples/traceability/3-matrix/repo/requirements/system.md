# LC-1 charge controller: system requirements

One requirement per heading. The statement is the paragraph under the heading,
written in one of the five EARS patterns; the attributes follow it. The
software requirements in [software.md](software.md) are derived from these.

## SYS-001: Record why charging stopped

The charger shall record each charge stop, with its cause, in the event log.

### verification: Test

### rationale: Rationale

A returned unit can be diagnosed only from what it recorded.

### trace: Traces to

[§STK-002](stakeholder-needs.md#stk-002-a-stopped-charger-stays-safe-and-says-why)

## SYS-002: Stop charging above the temperature limit

When the cell temperature rises above `45 °C` (§PAR-001), the charger shall stop
the charge current within `500 ms` (§PAR-002).

### verification: Test

### rationale: Rationale

The cell supplier's datasheet allows charging only up to this temperature. The
reaction time bounds how far the cell can heat after it crosses the limit.

### trace: Traces to

[§STK-001](stakeholder-needs.md#stk-001-charging-never-overheats-the-battery)

## SYS-003: Stay stopped after a fault

While the charger is in the fault state, the charger shall keep the charge
current off.

### verification: Test

### rationale: Rationale

A fault that cleared itself would restart charging with its cause unexplained.

### trace: Traces to

[§STK-002](stakeholder-needs.md#stk-002-a-stopped-charger-stays-safe-and-says-why)

## SYS-004: Stop when the temperature cannot be measured

If the cell temperature cannot be measured, then the charger shall stop the
charge current and enter the fault state.

### verification: Test

### rationale: Rationale

An open or shorted thermistor reads as a plausible temperature only by accident,
and charging blind defeats [§SYS-002](system.md#sys-002-stop-charging-above-the-temperature-limit).

### trace: Traces to

[§STK-001](stakeholder-needs.md#stk-001-charging-never-overheats-the-battery),
[§STK-002](stakeholder-needs.md#stk-002-a-stopped-charger-stays-safe-and-says-why)

## SYS-005: Show the fault on the status LED

Where the status LED is fitted, the charger shall blink the status LED while it
is in the fault state.

### verification: Demonstration

### rationale: Rationale

Variants without a display can show a fault only through the LED.

### trace: Traces to

[§STK-002](stakeholder-needs.md#stk-002-a-stopped-charger-stays-safe-and-says-why)
