//! Supplied paths fail before batch answers (§FS-show.2.6), while explicit
//! queries retain whole-project resolution and generated queries retain their
//! path selection (§FS-scan.path.show). Empty streams load nothing (§FS-show.2.6).

use super::{
    MALFORMED_INPUT, Repo, assert_empty_succeeded, assert_malformed_rejected, loads, observed,
    records, run_batch_in, stderr, stdout, write,
};
use grund_core::{BatchShowQuery, ShowFormat, ShowOpts, show_batch_with_scope};

const INPUT: &str = "{\"id\":\"FS-alpha\"}\n{\"id\":\"member/FS-beta\"}\n";
const FORMATS: [&str; 3] = ["text", "json", "md"];

fn assert_missing(output: &std::process::Output, path: &str) {
    assert_eq!(output.status.code(), Some(2), "{}", stderr(output));
    assert_eq!(stdout(output), "");
    assert_eq!(
        stderr(output),
        format!("error: path does not exist: {path}\n")
    );
}

#[test]
fn explicit_batches_refuse_missing_positional_and_flag_paths_in_every_format() {
    let repo = Repo::new("missing-batch-path");
    for format in FORMATS {
        for path in ["FS-gone", "docs/gone.md"] {
            for args in [vec![path], vec!["--path", path]] {
                assert_missing(&run_batch_in(repo.path(), format, &args, INPUT), path);
            }
        }
    }
}

#[test]
fn malformed_input_precedes_missing_path_refusal() {
    let repo = Repo::new("malformed-missing-path");
    for format in FORMATS {
        assert_malformed_rejected(&run_batch_in(
            repo.path(),
            format,
            &["--path", "FS-gone"],
            MALFORMED_INPUT,
        ));
    }
}

#[test]
fn empty_explicit_streams_ignore_missing_paths_and_invalid_configuration() {
    let repo = Repo::new("empty-missing-path");
    write(&repo.path().join("grund.toml"), "not valid TOML [");
    for format in FORMATS {
        for args in [&["FS-gone"][..], &["--path", "docs/gone.md"][..]] {
            assert_empty_succeeded(&run_batch_in(repo.path(), format, args, " \n\n"));
        }
    }
}

#[test]
fn existing_paths_preserve_explicit_answers_and_generated_selection() {
    let repo = Repo::new("existing-batch-path");
    std::fs::create_dir(repo.path().join("docs/narrow")).unwrap();
    write(
        &repo.path().join("docs/narrow/FS-scoped.md"),
        "# FS-scoped: Scoped\n\nScoped lead.\n",
    );
    for format in FORMATS {
        let baseline = run_batch_in(repo.path(), format, &[], INPUT);
        assert_eq!(baseline.status.code(), Some(0), "{}", stderr(&baseline));
        for args in [&["docs/narrow"][..], &["--path", "docs/narrow"][..]] {
            let narrowed = run_batch_in(repo.path(), format, args, INPUT);
            assert_eq!(narrowed.status.code(), Some(0), "{}", stderr(&narrowed));
            assert_eq!(stderr(&narrowed), "");
            assert_eq!(stdout(&narrowed), stdout(&baseline));
        }
    }
    for selection in [&["--all"][..], &["--glob", "FS-*"][..]] {
        let mut args = selection.to_vec();
        args.extend(["--path", "docs/narrow"]);
        let generated = run_batch_in(repo.path(), "json", &args, "");
        assert_eq!(generated.status.code(), Some(0), "{}", stderr(&generated));
        assert_eq!(stderr(&generated), "");
        let generated = records(&generated);
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0]["query"]["id"], "FS-scoped");
        assert!(generated[0]["query"]["section"].is_null());
        assert_eq!(generated[0]["ok"], true);
    }
}

fn queries() -> Option<Vec<BatchShowQuery>> {
    Some(vec![BatchShowQuery {
        id: "FS-alpha".into(),
        section: None,
    }])
}

fn opts(path: std::path::PathBuf, format: ShowFormat) -> ShowOpts {
    let mut opts = ShowOpts::default();
    opts.path = path;
    opts.format = format;
    opts
}

#[test]
fn api_batches_refuse_missing_paths_and_preserve_explicit_resolution() {
    let repo = Repo::new("api-batch-path");
    for format in [ShowFormat::Text, ShowFormat::Json] {
        for queries in [queries(), None] {
            let path = repo.path().join("FS-gone");
            let (_, result) = show_batch_with_scope(queries, opts(path.clone(), format), true);
            let error = result.err().expect("missing path refuses the run");
            assert_eq!(
                error.to_string(),
                format!("path does not exist: {}", path.display())
            );
        }
    }
    std::fs::create_dir(repo.path().join("docs/narrow")).unwrap();
    let (_, result) = show_batch_with_scope(
        queries(),
        opts(repo.path().join("docs/narrow"), ShowFormat::Text),
        true,
    );
    let records = result.expect("existing path still answers outside the path");
    assert_eq!(records.len(), 1);
    assert!(
        records[0]
            .result
            .as_ref()
            .unwrap()
            .body
            .contains("Alpha lead.")
    );

    write(&repo.path().join("grund.toml"), "not valid TOML [");
    let (warnings, result) = show_batch_with_scope(
        Some(Vec::new()),
        opts(repo.path().join("FS-gone"), ShowFormat::Text),
        true,
    );
    assert!(warnings.is_empty());
    assert!(
        result
            .expect("empty explicit batch loads nothing")
            .is_empty()
    );
}

/// Count only with the observer compiled into this executable (§AR-ci.3.3.1).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the workspace-load observer; run with --features grund/test-workspace-load-count"
)]
fn supplied_path_validation_keeps_one_load_and_empty_or_malformed_streams_load_none() {
    let repo = Repo::new("batch-path-loads");
    let missing = repo.load_log("missing");
    assert_missing(
        &observed::run_observed(repo.path(), "json", &["FS-gone"], INPUT, &missing),
        "FS-gone",
    );
    assert_eq!(loads(&missing), 1);
    let existing = repo.load_log("existing");
    let output = observed::run_observed(repo.path(), "json", &["--path", "docs"], INPUT, &existing);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(records(&output).len(), 2);
    assert_eq!(loads(&existing), 1);
    let empty = repo.load_log("empty");
    assert_empty_succeeded(&observed::run_observed(
        repo.path(),
        "json",
        &["--path", "FS-gone"],
        "",
        &empty,
    ));
    assert_eq!(loads(&empty), 0);
    let malformed = repo.load_log("malformed");
    assert_malformed_rejected(&observed::run_observed(
        repo.path(),
        "json",
        &["FS-gone"],
        MALFORMED_INPUT,
        &malformed,
    ));
    assert_eq!(loads(&malformed), 0);
}
