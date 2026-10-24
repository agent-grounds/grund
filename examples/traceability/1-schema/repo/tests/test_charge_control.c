/* Unit tests of the charge-control task, run on the host against fakes of the
 * board interface. Each test case is declared on the function that runs it. */
#include <assert.h>
#include "../src/charger.h"

static int switch_opened;
static int log_entries;
static enum stop_cause last_cause;
static bool led;

void charge_switch_open(void) { switch_opened = 1; }
void event_log_write(enum stop_cause cause) { log_entries++; last_cause = cause; }
bool led_get(void) { return led; }
void led_set(bool on) { led = on; }

static void setup(void)
{
    charge_control_reset();
    switch_opened = 0;
    log_entries = 0;
    led = false;
}

/**
 * TC-001: Charging stops one degree above the temperature limit
 *
 * Verifies §SWR-001 at its boundary: charging continues at the limit,
 * `45` (§PAR-001.1), and stops at 46.
 */
static void test_over_temperature_opens_switch(void)
{
    setup();
    charge_control_step(45);
    assert(!switch_opened);
    charge_control_step(46);
    assert(switch_opened);
    assert(charge_control_state() == STOPPED);
}

/**
 * TC-002: An over-temperature stop is logged with its cause
 *
 * Verifies §SWR-006.
 */
static void test_stop_is_logged(void)
{
    setup();
    charge_control_step(46);
    assert(log_entries == 1);
    assert(last_cause == CAUSE_OVER_TEMP);
}

/**
 * TC-003: An invalid temperature latches the fault
 *
 * Verifies §SWR-004, and §SWR-006 for the cause it logs.
 */
static void test_invalid_temperature_latches_fault(void)
{
    setup();
    charge_control_step(TEMP_INVALID);
    assert(switch_opened);
    assert(charge_control_state() == FAULT);
    assert(last_cause == CAUSE_SENSOR_FAULT);
}

/**
 * TC-004: The switch stays open while the fault is latched
 *
 * Verifies §SWR-005: a valid, cool reading after the fault changes nothing.
 */
static void test_fault_keeps_switch_open(void)
{
    setup();
    charge_control_step(TEMP_INVALID);
    switch_opened = 0;
    charge_control_step(25);
    assert(switch_opened);
    assert(charge_control_state() == FAULT);
}

#ifdef CONFIG_STATUS_LED
/**
 * TC-005: The LED blinks while the fault is latched
 *
 * Verifies §SWR-007 in a build with the status LED.
 */
static void test_led_blinks_in_fault(void)
{
    setup();
    indicator_step();
    assert(!led);
    charge_control_step(TEMP_INVALID);
    indicator_step();
    assert(led);
    indicator_step();
    assert(!led);
}
#endif

int main(void)
{
    test_over_temperature_opens_switch();
    test_stop_is_logged();
    test_invalid_temperature_latches_fault();
    test_fault_keeps_switch_open();
#ifdef CONFIG_STATUS_LED
    test_led_blinks_in_fault();
#endif
    return 0;
}
