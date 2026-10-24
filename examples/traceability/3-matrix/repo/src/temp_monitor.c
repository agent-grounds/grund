#include "charger.h"

/* Valid ADC range: `100` (§DES-001.1.1) to `3900` (§DES-001.1.2) */
#define ADC_VALID_MIN 100
#define ADC_VALID_MAX 3900

/**
 * Convert one thermistor ADC count to whole degrees Celsius, or to
 * TEMP_INVALID outside the valid range: §DES-001.
 */
int temp_monitor_read(uint16_t adc_count)
{
    if (adc_count < ADC_VALID_MIN || adc_count > ADC_VALID_MAX) {
        return TEMP_INVALID;
    }
    return (ADC_VALID_MAX - adc_count) / 40 - 10;
}
