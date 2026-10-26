/* Unit tests of the temperature monitor, run on the host. */
#include <assert.h>
#include "../src/charger.h"

/**
 * TC-006: A count outside the valid range reads as invalid
 *
 * Verifies §SWR-003 on both edges of the range, `100` (§DES-001.1.1) and
 * `3900` (§DES-001.1.2): each edge converts, one count past it does not.
 */
static void test_out_of_range_is_invalid(void)
{
    assert(temp_monitor_read(100) != TEMP_INVALID);
    assert(temp_monitor_read(99) == TEMP_INVALID);
    assert(temp_monitor_read(3900) != TEMP_INVALID);
    assert(temp_monitor_read(3901) == TEMP_INVALID);
}

int main(void)
{
    test_out_of_range_is_invalid();
    return 0;
}
