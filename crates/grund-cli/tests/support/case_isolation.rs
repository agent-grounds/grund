//! §FS-examples.5.6.1: overlapping real-runner calls own their mutable trees.
//! The reported symlink fixture keeps its original diagnostic goldens
//! (§FS-declarations.stubs.target.3). Scheduling lives in an external child,
//! using the ordinary manifest seam, with no hooks in runner implementation.

use super::case_runner::{self, CaseKind, CaseOutcome, assert_every_case_passed, run_case};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CASE: &str = "check-stub-unlisted-extension-symlink-target-duplicate";
const TEST: &str = "case_isolation::overlapping_mutable_cases_keep_their_own_copy";
const PROBE: &str = "GRUND_797_ISOLATION_PROBE";

// A ready file acknowledges actual child entry, after the real runner copied
// and linked the fixture. Only then does the parent start the second run.
// Polling waits for acknowledged events with a deadline; it is not race stress.
const CHILD: &str = r#"
import os, pathlib, subprocess, sys, time
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
root, control = map(pathlib.Path, sys.argv[1:3])
grund, copy_arg, suffix_arg = sys.argv[3:6]
repo = (root / copy_arg).resolve()
assert pathlib.Path.cwd().resolve() == repo, 'cwd and argv must name the same copy'
assert (root / suffix_arg).resolve() == repo / 'notes/a.md', 'suffix uses the same copy'
assert (repo / 'notes/a.zz').is_symlink(), 'the original symlink fixture must run'
try:
    (control / 'first.claim').mkdir()
    slot = 'first'
except FileExistsError:
    slot = 'second'
witness = repo / 'owner-witness.txt'
witness.write_text(slot)
(control / (slot + '.ready')).write_text(str(repo))
deadline = time.monotonic() + 15
while not (control / 'release').exists():
    if time.monotonic() >= deadline:
        raise RuntimeError('parent did not release the invocation within 15 seconds')
    time.sleep(0.01)
assert witness.read_text() == slot, 'another invocation overwrote my copy'
witness.unlink()
result = subprocess.run([grund, 'check', str(repo)])
(repo / 'finished.txt').write_text('finished\n')
if len(sys.argv) > 6:
    print('relative copy: ' + os.path.relpath(repo, root))
    print('absolute copy: ' + str(repo))
    print('unrelated: target/e2e-work/unrelated/repo')
sys.exit(result.returncode)
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .expect("scratch home");
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(home).join("ag/tmp").join(format!(
            "grund-797-isolation-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create isolated test corpus");
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn ready(control: &Path, slot: &str) -> PathBuf {
    let path = control.join(format!("{slot}.ready"));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(value) = fs::read_to_string(&path)
            && !value.is_empty()
        {
            return PathBuf::from(value);
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {slot} child entry"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

// Release on assertion failure too, before scoped threads are joined.
struct Release<'a>(&'a Path);

impl Drop for Release<'_> {
    fn drop(&mut self) {
        let _ = fs::write(self.0.join("release"), "go");
    }
}

/// §FS-examples.5.6.1: hold the first real invocation in its child while the
/// second prepares the same fixture. Check actual cwd/argv paths and witnesses,
/// then let both finish through original exit/stream goldens and a final-tree
/// golden. No assumed copy-path formula or probabilistic overlap is involved.
/// §FS-examples.5.6.2: a subsequent path-output run preserves the logical copy
/// prefix in relative and absolute output while leaving unrelated paths alone.
#[test]
#[ignore = "red until #797 invocation-owned copies; implement removes this"]
fn overlapping_mutable_cases_keep_their_own_copy() {
    // Refresh selection is process-wide. Run the probe with comparison selected
    // without changing the environment of other libtest threads.
    if std::env::var_os(PROBE).is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--include-ignored", "--nocapture"])
            .env(PROBE, "1")
            .env_remove("UPDATE_EXPECTED")
            .output()
            .expect("run isolated comparison probe");
        assert!(
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("running 1 test")
                && String::from_utf8_lossy(&output.stdout).contains(TEST),
            "ownership probe failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        // Forward through the real stream: libtest captures eprintln! on a
        // passing test, which would hide an unsupported-platform skip again.
        std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)
            .expect("forward explicit fixture skip accounting");
        return;
    }

    let fixture = Fixture::new();
    let root = &fixture.0;
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let case = root.join("tests/e2e/cases").join(CASE);
    copy_tree(&source_root.join("tests/e2e/cases").join(CASE), &case);
    let control = root.join("control");
    fs::create_dir(&control).unwrap();

    // Preserve the reproducer's sequential diagnostic and golden controls.
    for kind in [CaseKind::E2e, CaseKind::Example] {
        let outcome = run_case(root, &case, kind);
        if matches!(&outcome, CaseOutcome::Skipped { .. }) {
            // The shared reporter names/counts an unsupported Windows symlink
            // skip. It is explicitly not evidence that ownership was tested.
            assert_every_case_passed("ownership probe: unsupported fixture", &[outcome]);
            eprintln!("ownership overlap NOT EXERCISED: symlinks unsupported");
            return;
        }
        assert_every_case_passed("sequential original diagnostic", &[outcome]);
    }
    assert!(matches!(
        case_runner::assert_case_is_deterministic(root, &case),
        CaseOutcome::NotApplicable
    ));

    // Add a tree surface in scratch only: original diagnostic bytes stay intact.
    copy_tree(&case.join("repo"), &case.join("expected.repo"));
    fs::copy(
        case.join("repo/notes/a.md"),
        case.join("expected.repo/notes/a.zz"),
    )
    .unwrap();
    fs::write(case.join("expected.repo/finished.txt"), "finished\n").unwrap();
    fs::remove_file(case.join("command.args")).unwrap();
    fs::write(case.join("command.cwd"), "{repo_copy}\n").unwrap();
    let external = serde_json::json!([
        "python3",
        "-c",
        CHILD,
        root,
        control,
        "{grund}",
        "{repo_copy}",
        "{repo_copy}/notes/a.md"
    ]);
    fs::write(case.join("command.external"), external.to_string()).unwrap();

    std::thread::scope(|scope| {
        let release = Release(&control);
        let first = scope.spawn(|| run_case(root, &case, CaseKind::E2e));
        let first_repo = ready(&control, "first");
        let second = scope.spawn(|| run_case(root, &case, CaseKind::Example));
        let second_repo = ready(&control, "second");
        let first_witness = fs::read_to_string(first_repo.join("owner-witness.txt"));
        let second_witness = fs::read_to_string(second_repo.join("owner-witness.txt"));
        drop(release);
        let outcomes = [first.join().unwrap(), second.join().unwrap()];

        assert_ne!(
            first_repo, second_repo,
            "overlapping real-runner invocations share one mutable copy"
        );
        assert_eq!(
            first_witness.unwrap(),
            "first",
            "second preparation disturbed first copy"
        );
        assert_eq!(second_witness.unwrap(), "second");
        assert_every_case_passed("overlapping original diagnostic and final trees", &outcomes);
    });

    // Keep the original overlap's diagnostic goldens exact. A separate phase
    // then adds path output in scratch, covering normalization without rewriting
    // any committed golden or prescribing the runtime allocation's location.
    let mut path_external = external.as_array().unwrap().clone();
    path_external.push(serde_json::json!("paths"));
    fs::write(
        case.join("command.external"),
        serde_json::to_string(&path_external).unwrap(),
    )
    .unwrap();
    let original_stdout = fs::read_to_string(case.join("expected.stdout")).unwrap();
    let logical = PathBuf::from("target/e2e-work").join(CASE).join("repo");
    fs::write(
        case.join("expected.stdout"),
        format!(
            "{original_stdout}relative copy: {}\nabsolute copy: {}\nunrelated: target/e2e-work/unrelated/repo\n",
            logical.display(), root.join(&logical).display()
        ),
    )
    .unwrap();
    let outcome = run_case(root, &case, CaseKind::E2e);
    assert_every_case_passed("logical copy paths in shared output", &[outcome]);
}
