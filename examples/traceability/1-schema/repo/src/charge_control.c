#include "charger.h"

static enum charge_state state;

/** Start in CHARGING, as at power-on: §DES-002. */
void charge_control_reset(void)
{
    state = CHARGING;
}

/* Open the switch, log why, and move to the next state (§DES-002.1). */
static void stop(enum charge_state next, enum stop_cause cause)
{
    charge_switch_open();
    event_log_write(cause);
    state = next;
}

/** Run one control period on the temperature just read: §DES-002.1, §DES-002.2. */
void charge_control_step(int temp_c)
{
    if (state == FAULT) {
        charge_switch_open();
        return;
    }
    if (temp_c == TEMP_INVALID) {
        stop(FAULT, CAUSE_SENSOR_FAULT);
    } else if (state == CHARGING && temp_c > CHARGE_TEMP_MAX_C) {
        stop(STOPPED, CAUSE_OVER_TEMP);
    }
}

/** The current state, for the indicator and for tests: §DES-002. */
enum charge_state charge_control_state(void)
{
    return state;
}

#ifdef CONFIG_STATUS_LED
/** Blink the LED in FAULT and hold it off otherwise: §DES-003. */
void indicator_step(void)
{
    led_set(state == FAULT ? !led_get() : false);
}
#endif
