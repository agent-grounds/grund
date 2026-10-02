// A blank line between: the rate is `811 RSD/h`

// (§CONST-hourly-rate.1) per the tariff.
pub fn blank_line() {}

// A different comment prefix: the rate is `812 RSD/h`
/// (§CONST-hourly-rate.1) documents the next item.
pub fn different_prefix() {}

// A code line follows: the rate is `813 RSD/h`
pub fn code_line() {} // (§CONST-hourly-rate.1)
