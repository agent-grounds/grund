//! Isolated Git home/XDG inputs with native subscriptions (§FS-check.6.1.3).
use crate::support::*;

#[test]
fn watch_inventory_global_git_and_xdg_discovery() {
    let _serial = crate::support::serial();
    for scenario in ["global", "home-config", "xdg-config", "worktree"] {
        let sandbox = Fixture::new();
        std::fs::create_dir_all(sandbox.0.join("home")).unwrap();
        std::fs::create_dir_all(sandbox.0.join("xdg")).unwrap();
        let result = bounded_child(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "global_ignore::watch_inventory_git_home_child",
                    "--nocapture",
                ])
                .env("GRUND_WATCH_INVENTORY_CHILD", scenario)
                .env("HOME", sandbox.0.join("home"))
                .env("USERPROFILE", sandbox.0.join("home"))
                .env("XDG_CONFIG_HOME", sandbox.0.join("xdg")),
        );
        assert!(
            result.status.success(),
            "isolated {scenario}: {} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn watch_inventory_git_home_child() {
    let Ok(scenario) = std::env::var("GRUND_WATCH_INVENTORY_CHILD") else {
        return;
    };
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    std::fs::create_dir_all(f.0.join(".git/info")).unwrap();
    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    let xdg = std::path::PathBuf::from(std::env::var_os("XDG_CONFIG_HOME").unwrap());
    let excludes = home.join("global-excludes");
    if scenario != "global" {
        std::fs::write(&excludes, "src/main.rs\n").unwrap();
    }
    let mut h = Harness::start(&f, None, None);
    let before = h.completed();
    assert_eq!(before.status, 1);
    match scenario.as_str() {
        "global" => std::fs::write(xdg.join("git/ignore"), "src/main.rs\n")
            .or_else(|_| {
                std::fs::create_dir_all(xdg.join("git"))?;
                std::fs::write(xdg.join("git/ignore"), "src/main.rs\n")
            })
            .unwrap(),
        "home-config" => std::fs::write(
            home.join(".gitconfig"),
            format!("[core]\nexcludesFile = {}\n", excludes.display()),
        )
        .unwrap(),
        "xdg-config" => {
            std::fs::create_dir_all(xdg.join("git")).unwrap();
            std::fs::write(
                xdg.join("git/config"),
                format!("[core]\nexcludesFile = {}\n", excludes.display()),
            )
            .unwrap();
        }
        "worktree" => {
            std::fs::remove_dir_all(f.0.join(".git")).unwrap();
            let gitdir = home.join("worktree");
            std::fs::create_dir_all(&gitdir).unwrap();
            let common = home.join("common");
            std::fs::create_dir_all(common.join("info")).unwrap();
            std::fs::write(
                gitdir.join("commondir"),
                common.to_string_lossy().as_bytes(),
            )
            .unwrap();
            std::fs::write(common.join("info/exclude"), "src/main.rs\n").unwrap();
            f.write(".git", &format!("gitdir: {}\n", gitdir.display()));
        }
        _ => unreachable!(),
    }
    let after = f.ordinary();
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    if scenario == "worktree" {
        std::fs::write(home.join("common/info/exclude"), "").unwrap();
        h.matches(&before);
        h.stop(before.status);
    } else {
        h.stop(after.status);
    }
}
