//! The observed half of the batch contract: `grund show --batch`, or a single
//! read, run by this test executable re-spawned into the CLI's entry, so the
//! workspace-load and configuration-resolution counts come from the observers
//! this test's own featured build compiled in, never from an uplifted binary
//! another build of the package re-links (§AR-ci.3.3.1).

use std::path::Path;
use std::process::{Command, Output};

/// The child test the re-spawned executable runs, by its full libtest name.
const CHILD: &str = "observed::show_batch_observed_child";
/// The CLI arguments the child runs, one per line; absent, the child is a no-op.
const ARGS: &str = "GRUND_SHOW_BATCH_OBSERVED_ARGS";

/// Runs `grund show --batch --format=<format> <extra_args>` in `repo` with
/// `stdin` in the re-spawned child, logging each workspace load to `load_log`,
/// and returns its output with libtest's own header taken off stdout
/// (§AR-ci.3.3.1).
pub(crate) fn run_observed(
    repo: &Path,
    format: &str,
    extra_args: &[&str],
    stdin: &str,
    load_log: &Path,
) -> Output {
    let format = format!("--format={format}");
    let mut cli_args = vec!["show", "--batch", &format];
    cli_args.extend_from_slice(extra_args);
    let mut command = child_command(&cli_args);
    command.env("GRUND_TEST_WORKSPACE_LOAD_LOG", load_log);
    child_output(command, repo, stdin)
}

/// Runs `grund <cli_args>` in `dir` in the re-spawned child, logging each
/// configuration resolution to `resolution_log`, and returns its output with
/// libtest's own header taken off stdout (§AR-resolver.3.1.1, §AR-ci.3.3.1).
pub(crate) fn run_observed_resolutions(
    dir: &Path,
    cli_args: &[&str],
    resolution_log: &Path,
) -> Output {
    let mut command = child_command(cli_args);
    command.env("GRUND_TEST_CONFIG_RESOLUTION_LOG", resolution_log);
    child_output(command, dir, "")
}

/// This test executable, asked to run only the child below with `cli_args`.
fn child_command(cli_args: &[&str]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("this test executable"));
    // One test thread, so libtest names the child before running it and the
    // header it writes ahead of the CLI's stdout is exactly the one below.
    command
        .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
        .args(["--format=pretty", "--color=never"])
        .env(ARGS, cli_args.join("\n"));
    command
}

/// Runs the child in `dir` with `stdin`, and takes libtest's header off stdout.
fn child_output(command: Command, dir: &Path, stdin: &str) -> Output {
    let mut output = super::run_piped(command, dir, stdin);
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
