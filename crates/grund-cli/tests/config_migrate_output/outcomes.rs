//! Error and no-op output invariants (§FS-config-migrate.6.2,
//! §FS-config-migrate.6.6, §FS-config-migrate.7).

use super::*;

/// §FS-config-migrate.7: full cannot turn a load error into a write.
#[test]
#[ignore = "red until #907 --full preserves load failures; implement removes this"]
fn full_load_failure_preserves_message_and_original_tree() {
    let bad = Fixture::new("not valid TOML {");
    let before = snapshot(&bad.0);
    let normal = bad.migrate(&["--write"]);
    status(&normal, 2);
    let full = bad.migrate(&["--write", "--full"]);
    status(&full, 2);
    assert_eq!(text(&full.stderr), text(&normal.stderr));
    assert!(full.stdout.is_empty());
    assert_eq!(snapshot(&bad.0), before);
}

/// §FS-config-migrate.6.6: full cannot turn a no-op into a refresh or write,
/// including a change to file modification time.
#[test]
#[ignore = "red until #907 --full preserves already-v2 no-ops; implement removes this"]
fn full_noops_preserve_message_bytes_and_mtime() {
    let fixture = Fixture::new(V1);
    status(&fixture.migrate(&["--write"]), 0);
    let before = snapshot(&fixture.0);
    let modified = fs::metadata(fixture.0.join("grund.toml"))
        .unwrap()
        .modified()
        .unwrap();
    for write in [false, true] {
        let mut flags = vec!["--full"];
        if write {
            flags.push("--write");
        }
        let out = fixture.migrate(&flags);
        status(&out, 0);
        assert!(out.stdout.is_empty());
        assert_eq!(
            text(&out.stderr),
            "grund.toml is already version 2; nothing to migrate\n"
        );
        assert_eq!(snapshot(&fixture.0), before);
        assert_eq!(
            fs::metadata(fixture.0.join("grund.toml"))
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
    }
}

/// §FS-config-migrate.6.2: force an actual atomic-writer I/O error even as
/// root, without changing production code or depending on disk exhaustion.
#[cfg(unix)]
#[test]
#[ignore = "red until #907 --full preserves atomic-write failures; implement removes this"]
fn full_atomic_write_failure_keeps_original_and_leaves_no_temporary_file() {
    use std::os::unix::process::CommandExt;

    let fixture = Fixture::new(V1);
    let before = snapshot(&fixture.0);
    let modified = fs::metadata(fixture.0.join("grund.toml"))
        .unwrap()
        .modified()
        .unwrap();
    let mut expected_stderr = None;
    for full in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
        command
            .current_dir(&fixture.0)
            .args(["config", "migrate", "--to", "2", "--write"]);
        if full {
            command.arg("--full");
        }
        // SAFETY: these async-signal-safe libc calls only change the child's
        // file-size limit and signal disposition before exec, with no locks.
        unsafe {
            command.pre_exec(|| {
                if libc::signal(libc::SIGXFSZ, libc::SIG_IGN) == libc::SIG_ERR {
                    return Err(std::io::Error::last_os_error());
                }
                let limit = libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let out = command.output().unwrap();
        status(&out, 2);
        let stderr = text(&out.stderr);
        assert!(
            stderr.contains("could not write the migrated config")
                && stderr.contains("the original is unchanged"),
            "{stderr}"
        );
        assert!(!stderr.contains(REMINDER));
        assert!(out.stdout.is_empty());
        if let Some(expected) = &expected_stderr {
            assert_eq!(&stderr, expected);
        } else {
            expected_stderr = Some(stderr);
        }
        assert_eq!(snapshot(&fixture.0), before, "no temporary file remains");
        assert_eq!(
            fs::metadata(fixture.0.join("grund.toml"))
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
    }
}
