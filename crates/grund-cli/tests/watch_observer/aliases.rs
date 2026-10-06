//! Shared native ownership and lexical anchors (§FS-check.6.1.1, §FS-check.6.1.3).
#![cfg(any(unix, windows))]
use crate::support::*;
use grund::WatchObservation;
use std::{fs, path::Path};

fn link(target: &Path, path: &Path, directory: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let _ = directory;
        std::os::unix::fs::symlink(target, path)
    }
    #[cfg(windows)]
    {
        if directory {
            std::os::windows::fs::symlink_dir(target, path)
        } else {
            std::os::windows::fs::symlink_file(target, path)
        }
    }
}

/// §FS-check.6.1.1, §FS-check.6.1.3: retiring recursive parent coverage must
/// retain the newly selected descendant's native subscription and mode.
#[test]
fn watch_recursive_parent_retirement_preserves_child() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"tree\"]\n"),
    );
    f.write("tree/old.rs", BAD);
    f.write("tree/retained/main.rs", CLEAN);
    let before = f.ordinary();
    assert_eq!(before.status, 1);
    let mut h = Harness::start(&f, None, None);
    h.matches(&before);
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"tree/retained\"]\n"),
    );
    let after = f.ordinary();
    assert_eq!(after.status, 0);
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    f.write("tree/retained/main.rs", BAD);
    let changed = f.ordinary();
    assert_eq!(changed.status, 1);
    h.matches(&changed);
    f.write("tree/retained/main.rs", CLEAN);
    h.matches(&after);
    h.stop(0);
}

#[test]
fn watch_directory_alias_refresh_routes() {
    let _serial = crate::support::serial();
    for scenario in [
        "optional-member",
        "global-config-replacement",
        "directory-retarget",
    ] {
        let sandbox = Fixture::new();
        sandbox.write("real/config/gitconfig", "");
        sandbox.write("real/retained.rs", CLEAN);
        fs::create_dir_all(sandbox.0.join("home")).unwrap();
        fs::create_dir_all(sandbox.0.join("xdg")).unwrap();
        for (target, path, directory) in [
            (sandbox.0.join("real"), sandbox.0.join("alias"), true),
            (
                sandbox.0.join("alias/config/gitconfig"),
                sandbox.0.join("home/.gitconfig"),
                false,
            ),
        ] {
            if let Err(err) = link(&target, &path, directory) {
                #[cfg(windows)]
                if matches!(err.raw_os_error(), Some(1314)) {
                    eprintln!(
                        "directory-alias regression requires Windows symlink privilege: {err}"
                    );
                    return;
                }
                panic!("creating alias fixture: {err}");
            }
        }
        let result = bounded_child(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "aliases::watch_directory_alias_child",
                    "--nocapture",
                ])
                .env("GRUND_WATCH_ALIAS_CHILD", scenario)
                .env("GRUND_WATCH_ALIAS_SANDBOX", &sandbox.0)
                .env("HOME", sandbox.0.join("home"))
                .env("USERPROFILE", sandbox.0.join("home"))
                .env("XDG_CONFIG_HOME", sandbox.0.join("xdg")),
        );
        assert!(result.status.success(), "isolated {scenario}: {result:?}");
    }
}

#[test]
fn watch_directory_alias_child() {
    let Ok(scenario) = std::env::var("GRUND_WATCH_ALIAS_CHILD") else {
        return;
    };
    let base = std::path::PathBuf::from(std::env::var_os("GRUND_WATCH_ALIAS_SANDBOX").unwrap());
    let f = Fixture::new();
    // §FS-check.6.1.3: ignore reads process cwd; mutate it only in this --exact child.
    std::env::set_current_dir(&f.0).unwrap();
    fs::create_dir_all(f.0.join(".git")).unwrap();
    link(
        &base.join("real/retained.rs"),
        &f.0.join("src/retained.rs"),
        false,
    )
    .unwrap();
    let excludes = base.join("excluded");
    let config = format!("[core]\nexcludesFile = {}\n", excludes.display());
    if scenario == "optional-member" {
        f.write(
            "grund.toml",
            &format!("{CONFIG}\n[workspace]\nmembers = []\noptional_members = [\"optional\"]\n"),
        );
    } else {
        f.write("src/main.rs", BAD);
        fs::write(&excludes, "src/main.rs\n").unwrap();
        fs::write(base.join("real/config/gitconfig"), &config).unwrap();
    }
    let before = f.ordinary();
    assert_eq!(before.status, 0);
    let mut h = Harness::start(&f, None, None);
    h.matches(&before);
    if scenario == "optional-member" {
        f.write(
            "optional/grund.toml",
            &format!("project_name = \"optional\"\n{CONFIG}"),
        );
        f.write("optional/src/main.rs", "// \u{a7}FS-member-missing\n");
    } else if scenario == "global-config-replacement" {
        fs::write(base.join("home/next-config"), "").unwrap();
        fs::rename(base.join("home/next-config"), base.join("home/.gitconfig")).unwrap();
    } else {
        fs::create_dir_all(base.join("next-real/config")).unwrap();
        fs::write(base.join("next-real/config/gitconfig"), "").unwrap();
        link(&base.join("next-real"), &base.join("next-alias"), true).unwrap();
        #[cfg(windows)]
        fs::remove_dir(base.join("alias")).unwrap();
        fs::rename(base.join("next-alias"), base.join("alias")).unwrap();
    }
    let after = f.ordinary();
    assert_eq!(after.status, 1);
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    if scenario == "optional-member" {
        f.write("optional/src/main.rs", "// member implementation\n");
    } else if scenario == "directory-retarget" {
        // The newly followed config must remain observable after retargeting.
        fs::write(base.join("next-real/config/gitconfig"), &config).unwrap();
    } else {
        f.write("src/main.rs", CLEAN);
    }
    let repaired = f.ordinary();
    assert_eq!(repaired.status, 0);
    assert_ne!(after.stdout, repaired.stdout);
    h.matches(&repaired);
    // Retiring the config alias must not remove the real directory's retained
    // file-link coverage (§FS-check.6.1.3). Check native updates and repair.
    fs::write(base.join("real/retained.rs"), BAD).unwrap();
    let retained = f.ordinary();
    assert_eq!(retained.status, 1);
    assert_ne!(repaired.stdout, retained.stdout);
    h.matches(&retained);
    fs::write(base.join("real/retained.rs"), CLEAN).unwrap();
    h.matches(&repaired);
    h.stop(0);
    assert!(h.history.iter().any(|event| {
        matches!(event, WatchObservation::Subscribed(path, _) if path == &base.join("real"))
    }));
    assert!(!h.history.iter().any(|event| {
        matches!(event, WatchObservation::Subscribed(path, _) if path.starts_with(base.join("alias")))
    }));
}
