//! The guard on the base grund-lsp's fixture roots are built under: no config,
//! under either name, sits at or above a base it returns (§AR-ci.10.3).
//!
//! Every scratch tree here sits under a directory the case builds itself, and
//! no case sets `TMPDIR`: the process environment is shared by the cases
//! running beside it.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use support::{fixture_base, uncovered_base};

/// A fresh scratch directory for one case, under `/dev/shm` where Linux has it
/// so the tree a case plants configs in stays out of the system temp dir.
fn scratch(name: &str) -> PathBuf {
    let shm = Path::new("/dev/shm");
    let parent = if shm.is_dir() {
        shm.to_path_buf()
    } else {
        std::env::temp_dir()
    };
    let dir = parent.join(format!(
        "grund-lsp-fixture-base-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// The first config, under either name, at `dir` or above it.
fn covering_config(dir: &Path) -> Option<PathBuf> {
    let walk = fs::canonicalize(dir).expect("canonicalize");
    walk.ancestors().find_map(|level| {
        [level.join("grund.toml"), level.join(".agents/grund.toml")]
            .into_iter()
            .find(|candidate| candidate.is_file())
    })
}

/// `scratch/covered/<config_name>` above a candidate at `scratch/covered/nested/tmp`,
/// and a clean candidate at `scratch/clean/tmp`; returns (covered, clean, config).
fn covered_and_clean(scratch: &Path, config_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let config = scratch.join("covered").join(config_name);
    fs::create_dir_all(config.parent().expect("config parent")).expect("create config dir");
    fs::write(&config, "grund_config_version = 1\n").expect("write covering config");
    let covered = scratch.join("covered/nested/tmp");
    let clean = scratch.join("clean/tmp");
    fs::create_dir_all(&covered).expect("create covered candidate");
    fs::create_dir_all(&clean).expect("create clean candidate");
    assert_eq!(
        covering_config(&clean),
        None,
        "the clean candidate must start uncovered; move the scratch dir"
    );
    (covered, clean, config)
}

#[test]
fn a_candidate_a_bare_grund_toml_covers_is_skipped_for_a_clean_one() {
    let scratch = scratch("bare");
    let (covered, clean, config) = covered_and_clean(&scratch, "grund.toml");
    let base = uncovered_base(&[covered, clean.clone()]);
    assert_eq!(
        base,
        Ok(clean),
        "a base under {} must not be handed to a zero-config fixture",
        config.display()
    );
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn a_candidate_an_agents_grund_toml_covers_is_skipped_for_a_clean_one() {
    let scratch = scratch("agents");
    let (covered, clean, config) = covered_and_clean(&scratch, ".agents/grund.toml");
    let base = uncovered_base(&[covered, clean.clone()]);
    assert_eq!(
        base,
        Ok(clean),
        "a base under {} must not be handed to a zero-config fixture",
        config.display()
    );
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn with_every_candidate_covered_the_guard_names_tmpdir_and_the_covering_config() {
    let scratch = scratch("all-covered");
    let (covered, _clean, config) = covered_and_clean(&scratch, "grund.toml");
    let reason = uncovered_base(std::slice::from_ref(&covered))
        .expect_err("a covered candidate is no base for a zero-config fixture");
    for named in [
        "TMPDIR".to_string(),
        covered.display().to_string(),
        fs::canonicalize(&config)
            .expect("canonicalize config")
            .display()
            .to_string(),
    ] {
        assert!(
            reason.contains(&named),
            "the refusal must name {named}: {reason}"
        );
    }
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn no_config_sits_at_or_above_the_fixture_base() {
    let base = fixture_base();
    assert_eq!(
        covering_config(&base),
        None,
        "fixture_base returned {}, which a config covers",
        base.display()
    );
}
