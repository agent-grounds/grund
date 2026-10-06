//! Repository maintenance acceptance, through Git and the actual shell script.
//! Cleanup runs only in an owned disposable tree with a stubbed cargo command.
//! Local/remote gate context: §AR-ci.1.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn git(args: &[&str]) -> Output {
    let output = Command::new("git")
        .current_dir(repository_root())
        .args(["-c", "core.excludesFile=/dev/null"])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// §FS-repository-maintenance.1.1: the root is free of a script or wrapper.
#[test]
fn root_clean_script_is_absent() {
    assert!(
        matches!(
            fs::symlink_metadata(repository_root().join("clean.sh")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ),
        "repository-root clean.sh must be absent"
    );
}

/// §FS-repository-maintenance.1.1: executable mode survives the relocation.
#[test]
fn relocated_clean_script_is_tracked_executable_and_grounded() {
    let script = repository_root().join("scripts/clean.sh");
    let metadata = fs::symlink_metadata(&script)
        .expect("scripts/clean.sh must exist as an executable regular file");
    assert!(
        metadata.is_file(),
        "scripts/clean.sh must be a regular file"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(metadata.permissions().mode() & 0o111, 0);
    }
    let tracked = git(&["ls-files", "--stage", "--", "scripts/clean.sh"]);
    let entry = String::from_utf8(tracked.stdout).unwrap();
    assert!(
        entry.starts_with("100755 "),
        "expected Git mode 100755: {entry}"
    );
    let source = fs::read_to_string(script).unwrap();
    assert!(
        source.lines().any(|line| {
            line.trim_start().starts_with('#') && line.contains("§FS-repository-maintenance.1.4")
        }),
        "clean script must cite its cleanup responsibilities"
    );
}

/// §FS-repository-maintenance.1.2 and §FS-repository-maintenance.1.3:
/// check the supported manifest contract without installing or summoning ephor.
#[test]
fn checkout_manifest_binds_relocated_clean_at_root() {
    let source = fs::read_to_string(repository_root().join("ephor.json"))
        .expect("checkout-owned ephor.json must declare the relocated clean verb");
    let manifest: Value = serde_json::from_str(&source).expect("valid ephor.json");
    assert_eq!(manifest["clean"]["command"], "./scripts/clean.sh");
    assert_eq!(manifest["clean"]["cwd"], "root");
}

/// §FS-lsp.2.3: editor settings stay local; the plugin hint stays reachable.
#[test]
fn intellij_settings_are_untracked_and_setup_hint_remains() {
    let tracked = git(&["ls-files", "--", ".idea"]);
    assert!(
        tracked.stdout.is_empty(),
        ".idea/ must have no tracked files; found {}",
        String::from_utf8_lossy(&tracked.stdout).trim()
    );
    let guide = fs::read_to_string(repository_root().join("docs/user-facing/lsp.md"))
        .expect("existing editor setup guide");
    assert!(
        guide.contains("Install LSP4IJ"),
        "retain the LSP4IJ installation hint"
    );
}

/// §FS-lsp.2.3: global or .git/info/exclude rules cannot satisfy this policy.
#[test]
fn repository_gitignore_ignores_intellij_settings() {
    for path in [
        ".idea/externalDependencies.xml",
        ".idea/issue-444-probe/nested.xml",
    ] {
        let output = Command::new("git")
            .current_dir(repository_root())
            .args([
                "-c",
                "core.excludesFile=/dev/null",
                "check-ignore",
                "--no-index",
                "-v",
                path,
            ])
            .output()
            .expect("run git check-ignore");
        let detail = String::from_utf8(output.stdout).unwrap();
        assert!(
            output.status.success() && detail.starts_with(".gitignore:"),
            "repository .gitignore must ignore {path}; got {detail:?}"
        );
        let pattern = detail
            .splitn(3, ':')
            .nth(2)
            .unwrap()
            .split('\t')
            .next()
            .unwrap();
        assert!(
            !pattern.starts_with('!'),
            "ignore rule must not negate {path}"
        );
    }
}

#[cfg(unix)]
mod cleanup {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# cache directory\n";

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let script = fs::read(repository_root().join("scripts/clean.sh"))
                .expect("scripts/clean.sh must exist before testing its cleanup scope");
            static SERIAL: AtomicUsize = AtomicUsize::new(0);
            let base = PathBuf::from(std::env::var_os("HOME").expect("HOME")).join("ag/tmp");
            fs::create_dir_all(&base).unwrap();
            let root = base.join(format!(
                "grund-clean-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("claim a fresh fixture root");
            let fixture = Self(root);
            fixture.write("scripts/clean.sh", &script);
            fs::set_permissions(
                fixture.0.join("scripts/clean.sh"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            fixture.write("bin/cargo", b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"$PWD\" \"$@\" > cargo-call\nexit \"$CARGO_CLEAN_STATUS\"\n");
            fs::set_permissions(
                fixture.0.join("bin/cargo"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            fixture.write("Cargo.toml", b"[workspace]\n");
            for path in ["panta/build", "scripts/cache", ".git/cache"] {
                fixture.write(&format!("{path}/CACHEDIR.TAG"), TAG.as_bytes());
                fixture.write(&format!("{path}/payload"), b"cache data");
            }
            fixture.write("invalid/CACHEDIR.TAG", b"not a cache signature\n");
            fixture.write("untagged/payload", b"keep me");
            fixture.write(".git/config", b"git metadata");
            fixture
        }

        fn write(&self, path: &str, contents: &[u8]) {
            let file = self.0.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, contents).unwrap();
        }

        fn run(&self, cargo_status: &str) -> Output {
            self.run_with_path(cargo_status, &std::env::var_os("PATH").unwrap())
        }

        fn run_with_path(&self, cargo_status: &str, fallback: &std::ffi::OsStr) -> Output {
            let mut paths = vec![self.0.join("bin")];
            paths.extend(std::env::split_paths(fallback));
            Command::new(self.0.join("scripts/clean.sh"))
                .current_dir(&self.0)
                .env("PATH", std::env::join_paths(paths).unwrap())
                .env("CARGO_CLEAN_STATUS", cargo_status)
                .output()
                .expect("execute only the copied fixture script")
        }
    }

    /// §AR-ci.10.2: remove only the disposable root this fixture claimed.
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove owned cleanup fixture");
        }
    }

    /// §FS-repository-maintenance.1.3 and §FS-repository-maintenance.1.4:
    /// cargo and cache traversal keep their checkout-wide scope and protect Git.
    #[test]
    fn relocated_clean_preserves_checkout_cleanup_scope() {
        let fixture = Fixture::new();
        let output = fixture.run("0");
        assert_success(&fixture, &output);
    }

    fn assert_cargo_call(fixture: &Fixture) {
        assert_eq!(
            fs::read_to_string(fixture.0.join("cargo-call"))
                .expect("controlled cargo must record its invocation"),
            format!("{}\nclean\n", fixture.0.display())
        );
    }

    fn assert_success(fixture: &Fixture, output: &Output) {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_cargo_call(fixture);
        for path in ["panta/build", "scripts/cache"] {
            assert!(!fixture.0.join(path).exists(), "remove tagged {path}");
        }
        assert_preserved(fixture);
    }

    fn assert_preserved(fixture: &Fixture) {
        for (path, contents) in [
            ("invalid/CACHEDIR.TAG", "not a cache signature\n"),
            ("untagged/payload", "keep me"),
            (".git/config", "git metadata"),
            (".git/cache/CACHEDIR.TAG", TAG),
            (".git/cache/payload", "cache data"),
        ] {
            assert_eq!(fs::read_to_string(fixture.0.join(path)).unwrap(), contents);
        }
    }

    /// §FS-repository-maintenance.1.4: cargo failure prevents cache deletion.
    #[test]
    fn relocated_clean_stops_when_cargo_clean_fails() {
        let fixture = Fixture::new();
        assert_failure(&fixture, &fixture.run("7"));
    }

    fn assert_failure(fixture: &Fixture, output: &Output) {
        assert_eq!(output.status.code(), Some(7));
        assert_cargo_call(fixture);
        for path in [
            "panta/build/payload",
            "scripts/cache/payload",
            ".git/cache/payload",
        ] {
            assert_eq!(
                fs::read_to_string(fixture.0.join(path)).unwrap(),
                "cache data"
            );
        }
        for path in ["panta/build/CACHEDIR.TAG", "scripts/cache/CACHEDIR.TAG"] {
            assert_eq!(fs::read_to_string(fixture.0.join(path)).unwrap(), TAG);
        }
        assert_preserved(fixture);
    }

    #[cfg(target_os = "linux")]
    mod writer_pressure {
        use super::*;

        // Installed tools only: an echo decoy exposes cargo PATH fallback safely.
        fn isolated_path(fixture: &Fixture) -> PathBuf {
            let fallback = fixture.0.join("fallback");
            fs::create_dir(&fallback).unwrap();
            let search = std::env::var_os("PATH").unwrap();
            for (name, tool) in [
                ("sh", "sh"),
                ("find", "find"),
                ("head", "head"),
                ("grep", "grep"),
                ("rm", "rm"),
                ("dirname", "dirname"),
                ("cargo", "echo"),
            ] {
                let installed = std::env::split_paths(&search)
                    .map(|path| path.join(tool))
                    .find(|path| path.is_file())
                    .unwrap_or_else(|| panic!("missing installed test prerequisite: {tool}"));
                std::os::unix::fs::symlink(installed, fallback.join(name)).unwrap();
            }
            fallback
        }

        fn check(boundary: &str, cargo_status: &str) {
            let fixture = Fixture::new();
            let fallback = isolated_path(&fixture);
            let executable = fixture.0.join(boundary);
            let writer = match fs::OpenOptions::new().write(true).open(&executable) {
                Ok(writer) => Some(writer),
                // The cargo executable may disappear when the substitute becomes shell-local.
                Err(error)
                    if boundary == "bin/cargo" && error.kind() == std::io::ErrorKind::NotFound =>
                {
                    None
                }
                Err(error) => panic!("hold writable descriptor on {boundary}: {error}"),
            };
            if writer.is_some() {
                let error = Command::new(&executable)
                    .output()
                    .expect_err("Linux must reject direct execution while a writer is held");
                assert_eq!(error.raw_os_error(), Some(26));
                eprintln!("writer pressure at {boundary}: {error:?}");
            }
            let output = fixture.run_with_path(cargo_status, fallback.as_os_str());
            assert!(
                output.stdout.is_empty(),
                "escaped controlled cargo substitute to PATH decoy: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            match cargo_status {
                "0" => assert_success(&fixture, &output),
                "7" => assert_failure(&fixture, &output),
                _ => unreachable!(),
            }
            drop(writer);
        }

        /// §FS-repository-maintenance.1.4.1: scope survives a busy copied script.
        #[test]
        fn held_script_writer_preserves_cleanup_scope() {
            check("scripts/clean.sh", "0");
        }

        /// §FS-repository-maintenance.1.4.1: cargo status 7 survives a busy copied script.
        #[test]
        fn held_script_writer_preserves_cargo_failure() {
            check("scripts/clean.sh", "7");
        }

        /// §FS-repository-maintenance.1.4.1: scope and cargo isolation survive a busy stub.
        #[test]
        fn held_cargo_writer_preserves_cleanup_scope() {
            check("bin/cargo", "0");
        }

        /// §FS-repository-maintenance.1.4.1: the substitute still supplies status 7.
        #[test]
        fn held_cargo_writer_preserves_cargo_failure() {
            check("bin/cargo", "7");
        }
    }
}
