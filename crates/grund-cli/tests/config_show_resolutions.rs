//! `grund config show` prints everything, the v2 spelling included, from the one
//! configuration it resolved, so it resolves configuration once on a standalone
//! project, v1 or v2, as often as `grund list` does there (§AR-resolver.3.4).
//!
//! The test counts configuration resolutions through an observer only the
//! `test-workspace-load-count` feature compiles in, so a build without it ignores
//! the test, naming the feature (§AR-ci.3.3). It reads the count from this test
//! executable re-spawned into the CLI, never from `CARGO_BIN_EXE_grund`, which a
//! featureless build can re-link (§AR-ci.3.3.1). The bytes `config show` prints
//! are pinned where they run in every build: the golden cases
//! `config-show-v1-keeps-v1-defaults`, the v1 `warning:` line and v1 spelling, and
//! `config-show-v2-row-order`, the v2 spelling and an empty stderr.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// The batch's observed run belongs to the show contract; this file counts resolutions only.
#[allow(dead_code)]
#[path = "show_batch_contract/observed.rs"]
mod observed;

/// A fresh project under the repository's `target/`, holding `grund.toml` with
/// `config` and one declaration, `FS-root` in `docs/fs/root.md`: the shapes of the
/// issue's reproducer. Removed with the test's other files when it passes.
fn project(root: &Path, name: &str, config: &str) -> PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(dir.join("docs/fs")).expect("create the fixture's kind home");
    fs::write(dir.join("grund.toml"), config).expect("write the fixture's grund.toml");
    fs::write(dir.join("docs/fs/root.md"), "# FS-root: Root\n\nBody.\n")
        .expect("write the fixture's one declaration");
    dir
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// `grund config show` on a bare v2 project and on a v1 project with one kind
/// resolves configuration as often as `grund list` on the same project, once,
/// and never again to ask for the v2 spelling the `Config` it loaded already
/// carries (§AR-resolver.3.4). The count is the test-only observer's
/// (§AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the configuration-resolution observer; run with --features grund/test-workspace-load-count"
)]
fn config_show_resolves_configuration_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/config-show-resolutions")
        .join(std::process::id().to_string());
    let _ = fs::remove_dir_all(&root);
    let v2 = project(&root, "v2", "grund_config_version = 2\n");
    let v1 = project(
        &root,
        "v1",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n",
    );
    let run = |dir: &Path, name: &str, args: &[&str]| {
        let log = root.join(format!("{name}.resolutions"));
        let output = observed::run_observed_resolutions(dir, args, &log);
        (resolutions(&log), output)
    };
    let (v2_listed, v2_list_run) = run(&v2, "v2-list", &["list"]);
    let (v2_shown, v2_show_run) = run(&v2, "v2-show", &["config", "show"]);
    let (v1_listed, v1_list_run) = run(&v1, "v1-list", &["list"]);
    let (v1_shown, v1_show_run) = run(&v1, "v1-show", &["config", "show"]);

    // Each run is the answer it stands for, so no count compares a failure.
    let runs = [&v2_list_run, &v2_show_run, &v1_list_run, &v1_show_run];
    assert_eq!(
        runs.map(|run| run.status.code()),
        [Some(0); 4],
        "stderr was {:?}",
        runs.map(stderr)
    );
    let opening = runs.map(|run| stdout(run).lines().next().unwrap_or_default().to_owned());
    assert_eq!(
        [&opening[1], &opening[3]],
        ["grund_config_version = 2", "grund_config_version = 1"],
        "each `config show` prints its project's own spelling"
    );
    assert!(
        opening[2].starts_with("FS-root"),
        "the v1 `grund list` lists its one declaration; stdout was {:?}",
        stdout(&v1_list_run)
    );
    assert_eq!(
        [v2_listed, v1_listed],
        [1, 1],
        "the controls: `grund list` resolved configuration {v2_listed} times on the v2 \
         project and {v1_listed} on the v1 one; each is one load of one standalone \
         project, so the observer must see each resolve exactly once"
    );
    assert_eq!(
        [v2_shown, v1_shown],
        [v2_listed, v1_listed],
        "`grund config show` resolved configuration {v2_shown} times on the v2 project \
         and {v1_shown} on the v1 one, where `grund list` resolves {v2_listed} and \
         {v1_listed}: it resolved the configuration again to ask for the v2 spelling the \
         `Config` it loaded already carries (§AR-resolver.3.4)"
    );
    let _ = fs::remove_dir_all(&root);
}
