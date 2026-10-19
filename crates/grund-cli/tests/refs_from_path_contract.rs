//! End-to-end contract for project source resolution and path-bounded outgoing
//! sites (§FS-refs.from, §FS-scan.path.refs), ported from issue #857's reproducer.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-857-refs-from-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
        fs::create_dir_all(root.join("docs/sub")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("grund.toml"),
            "grund_config_version = 1\n\n\
             [reference]\nstrict = true\n\n\
             [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
             [scan]\ninclude = [\"docs\", \"src\"]\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/functional-spec/FS-a.md"),
            "# FS-a: A\n\nSee \u{a7}FS-missing and \u{a7}FS-login.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/functional-spec/FS-login.md"),
            "# FS-login: Login\n\n## format: Format\n\n### .1: Username\n\nText.\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "// See \u{a7}FS-login.format.1.\npub fn f() {}\n",
        )
        .unwrap();
        Self(root)
    }

    fn check(&self, cwd: &str, args: &[&str], expected: (i32, &str, &str)) -> Option<String> {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(self.0.join(cwd))
            .args(args)
            .output()
            .expect("run grund");
        let stdout = String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n");
        let stderr = String::from_utf8(output.stderr)
            .unwrap()
            .replace("\r\n", "\n");
        let actual = (
            output.status.code().unwrap(),
            stdout.as_str(),
            stderr.as_str(),
        );
        (actual != expected).then(|| {
            format!(
                "(cd {cwd}) grund {}\nexpected: {expected:?}\nactual: {actual:?}",
                args.join(" ")
            )
        })
    }

    fn assert(&self, cwd: &str, args: &[&str], expected: (i32, &str, &str)) {
        if let Some(failure) = self.check(cwd, args, expected) {
            panic!("{failure}");
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const A_SITES: &str = "docs/functional-spec/FS-a.md:3: \u{a7}FS-missing\n\
                      docs/functional-spec/FS-a.md:3: \u{a7}FS-login\n";

/// §FS-scan.path.show and §FS-refs.from.owned: the same source resolves in show,
/// and owns sites when the selected tree contains its declaration.
#[test]
fn refs_from_path_resolution_controls() {
    let fixture = Fixture::new();
    fixture.assert(
        "src",
        &["show", "FS-a", "."],
        (0, "See \u{a7}FS-missing and \u{a7}FS-login.\n", ""),
    );
    fixture.assert(
        "src",
        &["show", "FS-login", "--toc", "."],
        (0, "## format: Format\n### format.1: Username\n", ""),
    );
    fixture.assert("src", &["refs", "--from", "FS-a"], (0, A_SITES, ""));
    fixture.assert(
        "",
        &["refs", "--from", "FS-a", "docs/functional-spec"],
        (0, A_SITES, ""),
    );
    fixture.assert(
        "src",
        &["refs", "FS-login", "."],
        (0, "src/lib.rs:1: \u{a7}FS-login.format.1\n", ""),
    );
    fixture.assert("", &["refs", "--from", "FS-absent", "src"], (1, "", "ID not found: FS-absent\nhint: run `grund list` to see every declared ID, or `grund id <KIND> \"<title>\"` to propose a new one\n"));
}

/// §FS-refs.from: resolving the project source must not leak its out-of-path
/// owned sites; §FS-refs.from.records fixes the empty total's exact bytes.
#[test]
#[ignore = "red until #857 resolves the source in the owning project; implement removes this"]
fn refs_from_path_bare_source_outside_tree() {
    let fixture = Fixture::new();
    let mut failures = Vec::new();
    for (cwd, path) in [("", "src"), ("src", "."), ("docs/sub", ".")] {
        for total in [false, true] {
            let mut args = vec!["refs", "--from", "FS-a", path];
            if total {
                args.push("--total");
            }
            if let Some(failure) = fixture.check(
                cwd,
                &args,
                (0, if total { "cites nothing\n" } else { "" }, ""),
            ) {
                failures.push(failure);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.from: a valid section source uses the same project resolution.
#[test]
#[ignore = "red until #857 resolves the source in the owning project; implement removes this"]
fn refs_from_path_section_source_outside_tree() {
    let fixture = Fixture::new();
    let failures = [("", "src"), ("src", "."), ("docs/sub", ".")]
        .into_iter()
        .filter_map(|(cwd, path)| {
            fixture.check(
                cwd,
                &["refs", "--from", "FS-login.format", path],
                (0, "", ""),
            )
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

/// §FS-refs.from.stubs and §FS-scan.path.refs: reuse the unscanned-target
/// fixture to distinguish the stub's home, the site's file, and target homes.
#[test]
#[ignore = "red until #857 bounds owned stub sites by the path; implement removes this"]
fn refs_from_path_stub_outside_tree_with_inline_sites_inside() {
    let fixture = Fixture::new();
    let existing = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/refs-from-stub-unscanned-target/repo");
    fs::remove_dir_all(fixture.0.join("docs")).unwrap();
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    copy_tree(&existing, &fixture.0);
    // Keep every target resolvable: this contract is about site location,
    // not the different dangling-site rules for scanned and lent citations.
    fs::write(
        fixture.0.join("docs/fs/FS-ghost.md"),
        "# FS-ghost: Ghost\n\nA declared target.\n",
    )
    .unwrap();
    let expected = "src/inline.rs:3: \u{a7}FS-audit\nsrc/inline.rs:3: \u{a7}FS-ghost\nsrc/inline.rs:7: \u{a7}FS-ledger\n";
    fixture.assert("", &["refs", "--from", "FS-inline.1"], (0, expected, ""));
    fixture.assert(
        "",
        &["refs", "--from", "FS-inline.1", "src"],
        (0, expected, ""),
    );
    fixture.assert("", &["refs", "--from", "FS-inline.1", "docs"], (0, "", ""));
}
