/**
 * Interface of the LC-1 charge-control software, as §DES-002 and §DES-003
 * design it.
 */
#ifndef CHARGER_H
#define CHARGER_H

#include <stdbool.h>
#include <stdint.h>

/* Over-temperature limit, whole degrees Celsius: `45` (§PAR-001.1) */
#define CHARGE_TEMP_MAX_C 45

/* Control period, milliseconds: `100` (§PAR-003.1) */
#define CONTROL_PERIOD_MS 100

#define TEMP_INVALID (-1000)

enum charge_state { CHARGING, STOPPED, FAULT };
enum stop_cause { CAUSE_OVER_TEMP, CAUSE_SENSOR_FAULT };

/* Board interface, provided by the board support package. */
void charge_switch_open(void);
void event_log_write(enum stop_cause cause);
bool led_get(void);
void led_set(bool on);

int temp_monitor_read(uint16_t adc_count);
void charge_control_reset(void);
void charge_control_step(int temp_c);
enum charge_state charge_control_state(void);
void indicator_step(void);

#endif
