//! A `refs --from` source refused as `grund <ID>` refuses it (§FS-refs.from) takes
//! its format from the context the run loaded, where its hint lines are built, and
//! never resolves configuration again to choose it (§AR-resolver.3.1.1.1).
//!
//! The test counts configuration resolutions through an observer only the
//! `test-workspace-load-count` feature compiles in, so a build without it ignores
//! the test, naming the feature (§AR-ci.3.3). It reads the count from this test
//! executable re-spawned into the CLI, never from `CARGO_BIN_EXE_grund`, which a
//! featureless build can re-link (§AR-ci.3.3.1). The bytes of the same refusals are
//! pinned where they run in every build: the golden cases `refs-from-not-found`,
//! `refs-from-not-found-json` and `refs-from-invalid-id`, and
//! `tests/e2e/refs_from_refusal_format.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// The batch's observed run belongs to the show contract; this file counts resolutions only.
#[allow(dead_code)]
#[path = "show_batch_contract/observed.rs"]
mod observed;

/// The fixture of the golden case `workspace-show-path-breadcrumb`: a root project
/// declaring `FS-root` and a member `api`. Every run here only reads it.
fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/workspace-show-path-breadcrumb/repo")
}

/// What [`observed`] runs its child through: `command` in `dir`, its streams
/// captured. No run here reads stdin, so it is closed.
fn run_piped(mut command: Command, dir: &Path, stdin: &str) -> Output {
    assert!(
        stdin.is_empty(),
        "a resolution count feeds the CLI no stdin"
    );
    command
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("run the observed child")
}

/// Reads the configuration-resolution observer's log, one line per resolution a
/// run made (§AR-resolver.3.1.1). Only a test ignored without
/// `test-workspace-load-count` may call this: in a build without the feature
/// nothing writes the log, and an unwritten log reads as zero (§AR-ci.3.3).
fn resolutions(path: &Path) -> usize {
    fs::read_to_string(path).unwrap_or_default().lines().count()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A refused `refs --from` source given no `--format`, not found or invalid,
/// resolves configuration as often as the same command's success path, as
/// `grund FS-nope` refusing the same source, and as the refusal given `--format
/// text`, never once more (§AR-resolver.3.1.1.1). The count is the test-only
/// observer's (§AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the configuration-resolution observer; run with --features grund/test-workspace-load-count"
)]
#[cfg_attr(
    feature = "test-workspace-load-count",
    ignore = "red until #1423 takes the refused refs --from source's format from the loaded context; implement removes this"
)]
fn refs_from_refusal_resolves_configuration_once() {
    let logs = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("refs_from_refusal_resolutions")
        .join(std::process::id().to_string());
    let _ = fs::remove_dir_all(&logs);
    fs::create_dir_all(&logs).expect("create the resolution logs' directory");
    let run = |name: &str, args: &[&str]| {
        let log = logs.join(format!("{name}.resolutions"));
        let output = observed::run_observed_resolutions(&workspace(), args, &log);
        (resolutions(&log), output)
    };
    let (listed, listed_run) = run("listed", &["refs", "--from", "FS-root"]);
    let (show, show_run) = run("show", &["FS-nope"]);
    let (given, given_run) = run("given", &["refs", "--from", "FS-nope", "--format", "text"]);
    let (not_found, not_found_run) = run("not-found", &["refs", "--from", "FS-nope"]);
    let (invalid, invalid_run) = run("invalid", &["refs", "--from", "FS-"]);

    // Each run is the answer it stands for, so no count compares other failures.
    let runs = [
        &listed_run,
        &show_run,
        &given_run,
        &not_found_run,
        &invalid_run,
    ];
    assert_eq!(
        runs.map(|run| run.status.code()),
        [Some(0), Some(1), Some(1), Some(1), Some(1)],
        "stderr was {:?}",
        runs.map(stderr)
    );
    let opening = runs.map(|run| stderr(run).lines().next().unwrap_or_default().to_owned());
    assert_eq!(
        opening[1..],
        [
            "ID not found: FS-nope",
            "ID not found: FS-nope",
            "ID not found: FS-nope",
            "invalid ID `FS-`"
        ],
        "each refusal's first stderr line"
    );
    assert!(
        listed > 0 && listed == show && listed == given,
        "the controls resolved configuration {listed} (`refs --from FS-root`), {show} \
         (`grund FS-nope`) and {given} (`refs --from FS-nope --format text`) times; each \
         is one load, so the observer must see each resolve alike, and at least once"
    );
    assert_eq!(
        [not_found, invalid],
        [listed, listed],
        "`refs --from FS-nope` resolved configuration {not_found} times and `refs --from \
         FS-` {invalid}, where its success path, `grund FS-nope` and the refusal given \
         `--format text` resolve {listed}: the refusal resolved configuration again to \
         choose its format rather than reading the context the run loaded \
         (§AR-resolver.3.1.1.1)"
    );
    let _ = fs::remove_dir_all(&logs);
}
