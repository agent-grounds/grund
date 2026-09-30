// The refresh-mode contract of the case runner, pinned from outside the runner.
// Test-only, and in a file of its own rather than in `case_report.rs` where the
// other harness tests live: that file stands 69 code lines under its budget and
// the refresh implementation is going into a sibling of its own, so a corpus
// helper and three tests belong beside it rather than on top of it. Included
// into the same module, so the tests drive `run_case` and
// `assert_every_case_passed` themselves.

/// What a refresh pass owes: it writes the final tree
/// (§AR-workspace.9.1.5), rewrites nothing over an unchanged tree
/// (§AR-workspace.9.1.4), and says what it covered where a passing test's
/// output can still be read (§AR-workspace.9.5).
///
/// Every assertion is on observable state — bytes and modes under
/// `expected.repo`, and the pass's own piped output — never on a harness type,
/// so what is pinned here is the contract rather than the shape of the code
/// that will satisfy it.
#[cfg(test)]
mod refresh_tests {
    use super::{
        assert_every_case_passed, copy_dir, discover_e2e_cases, relative_files, run_case, CaseKind,
    };
    use std::collections::BTreeMap;
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::time::{Duration, SystemTime};

    /// The case every scratch corpus is built from: a `{repo_copy}` case whose
    /// `expected.repo` has a subdirectory and one file tracked `100755`, so the
    /// write has a mode to carry and a directory to empty. The scratch copy
    /// keeps the name, because a golden that embedded the case name would
    /// otherwise be wrong in the copy.
    const CASE: &str = "cover-missing-snapshot-counts";

    /// Names the corpus [`refresh_probe_runs_the_selected_corpus`] runs over.
    /// A refresh cannot be selected from inside the test that wants it —
    /// `UPDATE_EXPECTED` is process-wide and libtest runs tests in threads — so
    /// the pass runs in a re-exec of this same binary, the way the synthetic
    /// verdict probe already re-execs one (§FS-examples.5.1).
    const PROBE_ROOT: &str = "GRUND_REFRESH_PROBE_ROOT";

    const PROBE: &str = "case_runner::refresh_tests::refresh_probe_runs_the_selected_corpus";

    /// One fixed, distinctly old modification time, so "was this file written?"
    /// is answered exactly rather than by a filesystem's timestamp granularity:
    /// a write moves the stamp to now whatever the clock's resolution is.
    const STAMP: Duration = Duration::from_secs(1_000_000_000);

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Outside `target/e2e-harness-tests`, deliberately: `should_update_expected`
    /// switches refresh *off* beneath that root (§FS-examples.5.1), so a refresh
    /// corpus placed there would be a comparison run wearing a refresh test's
    /// name and would pass whether the refresh covers the tree or not.
    fn scratch_root(label: &str) -> PathBuf {
        repo_root()
            .join("target/e2e-refresh-tests")
            .join(format!("{label}-{}", std::process::id()))
    }

    /// Copy the source case into a scratch corpus of its own. No committed
    /// golden is mutated by anything in this file: every stale byte is staled in
    /// the copy.
    fn build_corpus(label: &str) -> PathBuf {
        let scratch = scratch_root(label);
        let _ = fs::remove_dir_all(&scratch);
        copy_dir(
            &repo_root().join("tests/e2e/cases").join(CASE),
            &scratch.join("tests/e2e/cases").join(CASE),
        );
        scratch
    }

    fn case_dir(scratch: &Path) -> PathBuf {
        scratch.join("tests/e2e/cases").join(CASE)
    }

    /// One pass over one scratch corpus, in a process of its own. Deliberately
    /// without `--nocapture`: an accounting only a `--nocapture` run can read is
    /// the defect §AR-workspace.9.5 exists to remove, so the pipe here is
    /// exactly the stream libtest's capture would have swallowed.
    fn pass(scratch: &Path, refresh: bool) -> Output {
        let mut command = Command::new(std::env::current_exe().expect("current e2e test binary"));
        command
            .args(["--exact", PROBE])
            .env(PROBE_ROOT, scratch)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if refresh {
            command.env("UPDATE_EXPECTED", "1");
        } else {
            command.env_remove("UPDATE_EXPECTED");
        }
        command.output().expect("run the refresh probe")
    }

    fn rendered(output: &Output) -> String {
        format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    }

    /// The anchor that keeps a failure diagnostic. A piped stderr with no
    /// accounting in it reads exactly like a child that never ran, so every test
    /// below first establishes that the pass happened and that its verdict came
    /// back through the pipes — on stdout, which is where libtest puts its own
    /// words, and which a refresh pass carries today as well as after the
    /// change. A failure then says "the accounting is absent", never "the pass
    /// said nothing".
    fn assert_probe_ran(output: &Output) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "the probe pass must succeed\n{}",
            rendered(output)
        );
        assert!(
            stdout.contains(PROBE) && stdout.contains("test result: ok"),
            "the probe pass did not report a verdict, so its pipes carried nothing\n{}",
            rendered(output)
        );
    }

    /// Age the tree golden by one word, and add a golden file the run does not
    /// produce in a directory that holds nothing else. Returns the bytes the run
    /// does produce for the aged file, which is what a refresh must put back.
    fn stale_the_tree_golden(case: &Path) -> String {
        let guide = case.join("expected.repo/docs/guide.md");
        let produced = fs::read_to_string(&guide).expect("read the tree golden");
        let aged = produced.replace("rollout", "rollback");
        assert_ne!(
            aged, produced,
            "the source case no longer holds the word this ages"
        );
        fs::write(&guide, &aged).expect("age the tree golden");
        let stray = case.join("expected.repo/stray");
        fs::create_dir_all(&stray).expect("create the stray golden's directory");
        fs::write(stray.join("superfluous.md"), "the run does not produce this\n")
            .expect("write the stray golden");
        produced
    }

    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).expect("stat golden").permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    fn is_executable(_path: &Path) -> bool {
        true
    }

    /// Read every golden of `case` and stamp each one to [`STAMP`], so a pass
    /// that writes a file is visible as a moved modification time.
    fn stamp_goldens(case: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let tree = case.join("expected.repo");
        let mut paths = vec![
            case.join("expected.exit"),
            case.join("expected.stdout"),
            case.join("expected.stderr"),
        ];
        paths.extend(relative_files(&tree).iter().map(|rel| tree.join(rel)));
        let stamp = SystemTime::UNIX_EPOCH + STAMP;
        let mut goldens = BTreeMap::new();
        for path in paths {
            let bytes =
                fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            File::options()
                .write(true)
                .open(&path)
                .and_then(|file| file.set_modified(stamp))
                .unwrap_or_else(|err| panic!("stamp {}: {err}", path.display()));
            goldens.insert(path, bytes);
        }
        goldens
    }

    /// Not a contract of its own: the entry point [`pass`] re-execs, so that a
    /// refresh runs in a process whose entire corpus is one scratch case.
    /// Without the selector it does nothing, which is what it does in the
    /// ordinary suite.
    #[test]
    fn refresh_probe_runs_the_selected_corpus() {
        let Some(root) = std::env::var_os(PROBE_ROOT) else {
            return;
        };
        let root = PathBuf::from(root);
        let outcomes = discover_e2e_cases(&root)
            .iter()
            .map(|case| run_case(&root, case, CaseKind::E2e))
            .collect::<Vec<_>>();
        assert_every_case_passed("e2e cases", &outcomes);
    }

    /// §AR-workspace.9.1.5: a refresh writes the tree golden, byte-for-byte and
    /// mode included, and makes its file set equal to the produced tree's — so
    /// the compare pass that follows it confirms rather than contradicts
    /// (§AR-workspace.9.1.4).
    #[test]
    fn refresh_writes_a_stale_tree_golden_and_removes_what_the_run_no_longer_produces() {
        let scratch = build_corpus("writes-a-stale-tree-golden");
        let case = case_dir(&scratch);
        let produced = stale_the_tree_golden(&case);
        let guide = case.join("expected.repo/docs/guide.md");
        let stray = case.join("expected.repo/stray");
        let fetcher = case.join("expected.repo/scripts/fetch-ticket");
        assert!(
            is_executable(&fetcher),
            "the source case's executable golden is the fixture this asserts on"
        );

        let output = pass(&scratch, true);
        assert_probe_ran(&output);

        assert_eq!(
            fs::read_to_string(&guide).expect("read the tree golden"),
            produced,
            "the refresh left the stale tree golden exactly as it found it\n{}",
            rendered(&output)
        );
        assert!(
            !stray.join("superfluous.md").exists(),
            "the refresh kept a golden file the run does not produce\n{}",
            rendered(&output)
        );
        assert!(
            !stray.exists(),
            "the directory that removal emptied was left behind\n{}",
            rendered(&output)
        );
        assert!(
            case.join("expected.repo").is_dir(),
            "the expected.repo root must survive a removal, even an emptying one"
        );
        assert!(
            is_executable(&fetcher),
            "the write must carry the produced file's mode"
        );

        let compare = pass(&scratch, false);
        assert!(
            compare.status.success(),
            "a compare pass run immediately after a refresh must pass\n{}",
            rendered(&compare)
        );

        let _ = fs::remove_dir_all(&scratch);
    }

    /// §AR-workspace.9.5: the pass names the surfaces it covered, the case count
    /// it covered them over, and every golden file it wrote or removed — on a
    /// stream a passing test's reader still gets, which is why the accounting is
    /// read off a pipe from a run without `--nocapture`.
    #[test]
    fn refresh_says_what_it_covered_where_a_passing_test_can_be_read() {
        let scratch = build_corpus("says-what-it-covered");
        let case = case_dir(&scratch);
        stale_the_tree_golden(&case);

        let output = pass(&scratch, true);
        assert_probe_ran(&output);
        let stderr = String::from_utf8_lossy(&output.stderr);

        for said in [
            "refreshed",
            "1 case(s)",
            "exit",
            "stdout",
            "stderr",
            "tree",
            "1 file(s) written, 1 removed",
            CASE,
            "guide.md",
            "superfluous.md",
        ] {
            assert!(
                stderr.contains(said),
                "the refresh pass never said {said:?}\n{}",
                rendered(&output)
            );
        }

        let _ = fs::remove_dir_all(&scratch);
    }

    /// §AR-workspace.9.1.4, extended to the tree: every surface is
    /// compare-then-write, so a refresh over an unchanged corpus rewrites no
    /// golden — twice over — and its accounting says so in as many words.
    #[test]
    fn refresh_over_an_unchanged_tree_writes_no_bytes() {
        let scratch = build_corpus("unchanged-tree");
        let case = case_dir(&scratch);
        let stamp = SystemTime::UNIX_EPOCH + STAMP;

        for round in 1..=2 {
            let goldens = stamp_goldens(&case);
            let output = pass(&scratch, true);
            assert_probe_ran(&output);

            for (path, bytes) in &goldens {
                assert_eq!(
                    &fs::read(path).expect("read golden"),
                    bytes,
                    "pass {round} changed the bytes of {} over an unchanged tree",
                    path.display()
                );
                let modified = fs::metadata(path)
                    .expect("stat golden")
                    .modified()
                    .expect("golden modification time");
                assert_eq!(
                    modified,
                    stamp,
                    "pass {round} rewrote {} over an unchanged tree\n{}",
                    path.display(),
                    rendered(&output)
                );
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("0 file(s) written, 0 removed"),
                "pass {round} never said it changed nothing\n{}",
                rendered(&output)
            );
        }

        let _ = fs::remove_dir_all(&scratch);
    }
}
