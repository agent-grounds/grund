//! Issue #1177's cases, run alone: a file-home write measures the home in whole lines, so an
//! indent stays on the line it opens and nothing is left behind as a whitespace-only line
//! (§FS-fetch.4.1). Each expectation is the column-0 twin's result, the same home with the
//! indent removed, with the indent put back on the line that had it. The e2e cases are held out
//! of the full pass until the fix lands, and implement removes the hold and the markers.

use std::path::Path;
use std::process::Command;

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case_inspecting};
use super::repo_root;

/// Red until #1177: the full pass skips these and the contract runs them; implement removes this.
pub const RED_UNTIL_1177: [&str; 7] = [
    "fetch-line-boundary-two-space-sibling",
    "fetch-line-boundary-two-space-sibling-no-blank-line",
    "fetch-line-boundary-insert-before-two-space-sibling",
    "fetch-line-boundary-indented-declaration-replaced",
    "fetch-line-boundary-four-space-line-kept",
    "fetch-line-boundary-tab-line-kept",
    "fetch-line-boundary-unchanged-output-two-space-sibling",
];

/// `grund check`'s exit in `repo`.
fn check_exit(repo: &Path) -> Option<i32> {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .current_dir(repo)
        .output()
        .expect("spawn grund check")
        .status
        .code()
}

/// The tree is green before the fetch and after it, as the column-0 twin's is, and the home
/// comes out at its golden bytes.
#[test]
#[ignore = "red until #1177 starts a file-home boundary at its line's first byte; implement removes this"]
fn fetch_line_boundaries_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let mut red_checks = Vec::new();
    let outcomes = RED_UNTIL_1177
        .iter()
        .map(|name| {
            let case = cases.join(name);
            if check_exit(&case.join("repo")) != Some(0) {
                red_checks.push(format!("{name}: not green before the fetch"));
            }
            run_case_inspecting(&root, &case, E2e, |repo| {
                if check_exit(repo) != Some(0) {
                    red_checks.push(format!("{name}: not green after the fetch"));
                }
            })
        })
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch line boundaries", &outcomes);
    assert!(red_checks.is_empty(), "{}", red_checks.join("\n"));
}

/// The shapes that want exact bytes: `.gitattributes` holds the e2e corpus to LF, so a CRLF home
/// cannot be a fixture there. The others are further shapes of the same write, a table row each
/// rather than a case directory. The fetcher is a POSIX script, as in a `requires-unix-shell` case.
#[cfg(unix)]
mod exact_bytes {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::check_exit;

    const CONFIG: &str = "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n\
                          [[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n\
                          fetch = \"scripts/fetch\"\n\n\
                          [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n";

    struct Shape {
        name: &'static str,
        /// The declaration the fetcher prints, as `<ID>: <title>`.
        printed: &'static str,
        home: &'static str,
        /// The start of the line that keeps its indent, as indented; `None` where the only
        /// indented line is the one replaced.
        kept: Option<&'static str>,
        expected: &'static str,
    }

    const SHAPES: [Shape; 6] = [
        Shape {
            name: "crlf-two-space-sibling",
            printed: "AR-001-thing: Thing",
            home: "## AR-001-thing: Thing\r\nold\r\n\r\n  ## AR-002-other: Other\r\nother\r\n",
            kept: Some("  ## AR-002-other"),
            expected: "## AR-001-thing: Thing\nnew\n\r\n  ## AR-002-other: Other\r\nother\r\n",
        },
        Shape {
            name: "crlf-two-space-sibling-no-blank-line",
            printed: "AR-001-thing: Thing",
            home: "## AR-001-thing: Thing\r\nold\r\n  ## AR-002-other: Other\r\nother\r\n",
            kept: Some("  ## AR-002-other"),
            expected: "## AR-001-thing: Thing\nnew\n\r\n  ## AR-002-other: Other\r\nother\r\n",
        },
        Shape {
            name: "three-space-sibling",
            printed: "AR-001-thing: Thing",
            home: "## AR-001-thing: Thing\nold\n\n   ## AR-002-other: Other\nother\n",
            kept: Some("   ## AR-002-other"),
            expected: "## AR-001-thing: Thing\nnew\n\n   ## AR-002-other: Other\nother\n",
        },
        Shape {
            name: "indented-declaration-first-in-the-file",
            printed: "AR-001-thing: Thing",
            home: "  ## AR-001-thing: Thing\nold\n\n## AR-002-other: Other\nother\n",
            kept: None,
            expected: "## AR-001-thing: Thing\nnew\n\n## AR-002-other: Other\nother\n",
        },
        Shape {
            name: "insert-first-in-the-file-before-two-space-sibling",
            printed: "AR-002-other: Other",
            home: "  ## AR-003-third: Third\nthird\n",
            kept: Some("  ## AR-003-third"),
            expected: "## AR-002-other: Other\nnew\n\n  ## AR-003-third: Third\nthird\n",
        },
        Shape {
            name: "insert-before-two-space-sibling-no-blank-line",
            printed: "AR-002-other: Other",
            home: "## AR-001-thing: Thing\none\n  ## AR-003-third: Third\nthird\n",
            kept: Some("  ## AR-003-third"),
            expected: "## AR-001-thing: Thing\none\n\n\
                       ## AR-002-other: Other\nnew\n\n  ## AR-003-third: Third\nthird\n",
        },
    ];

    /// One throwaway project, removed however the test ends.
    struct Project(PathBuf);

    impl Drop for Project {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    impl Project {
        fn new(printed: &str, home: &str) -> Self {
            static SERIAL: AtomicUsize = AtomicUsize::new(0);
            let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "grund-1177-line-boundary-{}-{serial}",
                std::process::id()
            ));
            let project = Self(root);
            fs::create_dir_all(project.0.join("docs")).expect("create docs");
            fs::create_dir_all(project.0.join("scripts")).expect("create scripts");
            fs::write(project.0.join("grund.toml"), CONFIG).expect("write config");
            fs::write(project.0.join("docs/arch.md"), home).expect("write home");
            let fetcher = project.0.join("scripts/fetch");
            let script = format!("#!/bin/sh\nset -eu\n\ncat <<'OUT'\n## {printed}\nnew\nOUT\n");
            fs::write(&fetcher, script).expect("write fetcher");
            fs::set_permissions(&fetcher, fs::Permissions::from_mode(0o755)).expect("chmod");
            project
        }

        fn repo(&self) -> &Path {
            &self.0
        }

        fn home(&self) -> String {
            fs::read_to_string(self.0.join("docs/arch.md")).expect("read home")
        }
    }

    /// Fetch into `home`, saying what is wrong with the run rather than panicking on it, so a
    /// failure names every shape: the bytes on success, whichever run was not green otherwise.
    fn fetched(name: &str, printed: &str, home: &str) -> Result<String, String> {
        let project = Project::new(printed, home);
        if check_exit(project.repo()) != Some(0) {
            return Err(format!("{name}: not green before the fetch"));
        }
        let id = printed.split(':').next().expect("an ID ahead of the colon");
        let fetch = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["fetch", id])
            .current_dir(project.repo())
            .output()
            .expect("spawn grund fetch");
        if !fetch.status.success() {
            return Err(format!(
                "{name}: fetch exited {:?}: {}",
                fetch.status.code(),
                String::from_utf8_lossy(&fetch.stderr)
            ));
        }
        if check_exit(project.repo()) != Some(0) {
            return Err(format!("{name}: not green after the fetch"));
        }
        Ok(project.home())
    }

    /// `home` with every heading line's indent removed, whatever its line ending.
    fn column_0_twin(home: &str) -> String {
        home.split_inclusive('\n')
            .map(|line| {
                let trimmed = line.trim_start_matches([' ', '\t']);
                if trimmed.starts_with("## ") {
                    trimmed
                } else {
                    line
                }
            })
            .collect()
    }

    fn shape_failures(shape: &Shape) -> Vec<String> {
        let mut failures = Vec::new();
        // The oracle first: a twin that stopped giving the expected bytes moved the premise,
        // and says nothing about the indent.
        let twin_name = format!("{} (column-0 twin)", shape.name);
        match fetched(&twin_name, shape.printed, &column_0_twin(shape.home)) {
            Ok(twin) => {
                let indented = shape.kept.map_or(twin.clone(), |kept| {
                    twin.replacen(kept.trim_start_matches([' ', '\t']), kept, 1)
                });
                if indented != shape.expected {
                    failures.push(format!(
                        "{twin_name}: with its indent back gives {indented:?}, expected {:?}",
                        shape.expected
                    ));
                }
            }
            Err(failure) => failures.push(failure),
        }
        match fetched(shape.name, shape.printed, shape.home) {
            Ok(home) if home == shape.expected => {}
            Ok(home) => failures.push(format!(
                "{}: home came out {home:?}, expected {:?}",
                shape.name, shape.expected
            )),
            Err(failure) => failures.push(failure),
        }
        failures
    }

    #[test]
    #[ignore = "red until #1177 starts a file-home boundary at its line's first byte; implement removes this"]
    fn fetch_line_boundaries_exact_bytes() {
        let failures = SHAPES.iter().flat_map(shape_failures).collect::<Vec<_>>();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
