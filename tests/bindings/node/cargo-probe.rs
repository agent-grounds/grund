//! Test-only native launcher for §FS-distribution.3.2.3.3 Cargo interception.

use std::{env, process};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script = env::current_exe()?.with_extension("py");
    let status = process::Command::new(
        env::var_os("GRUND_TEST_PROBE_PYTHON").ok_or("probe Python executable missing")?,
    )
    .arg(script)
    .args(env::args_os().skip(1))
    .status()?;
    process::exit(status.code().unwrap_or(1));
}
