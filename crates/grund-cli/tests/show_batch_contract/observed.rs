//! The observed half of the batch contract: `grund show --batch` run by this
//! test executable re-spawned into the CLI's entry, so the workspace-load count
//! comes from the observer this test's own featured build compiled in, never
//! from an uplifted binary another build of the package re-links (§AR-ci.3.3.1).

use std::path::Path;
use std::process::{Command, Output};

/// The child test the re-spawned executable runs, by its full libtest name.
const CHILD: &str = "observed::show_batch_observed_child";
/// The CLI arguments the child runs, one per line; absent, the child is a no-op.
const ARGS: &str = "GRUND_SHOW_BATCH_OBSERVED_ARGS";

/// Runs `grund show --batch --format=json <extra_args>` in `repo` with `stdin`
/// in the re-spawned child, logging each workspace load to `load_log`, and
/// returns its output with libtest's own header taken off stdout (§AR-ci.3.3.1).
pub(crate) fn run_observed(
    repo: &Path,
    extra_args: &[&str],
    stdin: &str,
    load_log: &Path,
) -> Output {
    let mut cli_args = vec!["show", "--batch", "--format=json"];
    cli_args.extend_from_slice(extra_args);
    let mut command = Command::new(std::env::current_exe().expect("this test executable"));
    // One test thread, so libtest names the child before running it and the
    // header it writes ahead of the CLI's stdout is exactly the one below.
    command
        .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
        .args(["--format=pretty", "--color=never"])
        .env(ARGS, cli_args.join("\n"))
        .env("GRUND_TEST_WORKSPACE_LOAD_LOG", load_log);
    let mut output = super::run_piped(command, repo, stdin);
    let header = format!("\nrunning 1 test\ntest {CHILD} ... ");
    assert!(
        output.stdout.starts_with(header.as_bytes()),
        "the re-spawned child did not open with libtest's header {header:?}; \
         status {:?}, stdout {:?}, stderr {:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.drain(..header.len());
    output
}

/// The re-spawned child: with its arguments set it runs the CLI's entry in this
/// process and exits with the CLI's own code, before libtest writes a result
/// line; without them it passes having done nothing (§AR-ci.3.3.1).
#[cfg(feature = "test-workspace-load-count")]
#[test]
fn show_batch_observed_child() {
    use std::io::Write;
    use std::process::ExitCode;

    let Some(args) = std::env::var_os(ARGS) else {
        return;
    };
    let args = args.to_str().expect("observed CLI arguments are UTF-8");
    let code = grund::main_entry_from_args(args.split('\n').map(String::from).collect());
    std::io::stdout().flush().expect("flush the CLI's stdout");
    // `ExitCode` has no stable accessor or exit; the CLI's codes are all `u8`s.
    let status = (0..=u8::MAX)
        .find(|&status| ExitCode::from(status) == code)
        .expect("the CLI exits with a u8 code");
    std::process::exit(i32::from(status));
}
