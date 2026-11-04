//! #1146's reproducer, ported to the CLI harness: `config migrate`'s
//! `differs:` comparison lists a run warning once, as `grund check
//! --suggestions` on the migrated tree prints it (§FS-config-migrate.5.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const UNREAD: &str = "+ run-warning warning unread-workspace-block ";

struct Fixture(PathBuf);

impl Fixture {
    /// A v2 root over a v1 `include_root = false` grouping block, `group`.
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1146-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let write = |path: &str, text: &str| {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write(
            "grund.toml",
            "grund_config_version = 2\n\n[workspace]\nmembers = [\"group\"]\n",
        );
        write(
            "group/grund.toml",
            "[id]\nnamed_sections = true\n\n[scan]\ninclude = []\n\n\
             [workspace]\ninclude_root = false\nmembers = [\"a\"]\n",
        );
        write("group/a/grund.toml", "grund_config_version = 2\n");
        write("group/README.md", "Cites \u{a7}FS-x.\n");
        Self(root)
    }

    fn group(&self) -> PathBuf {
        self.0.join("group")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn grund(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// §FS-config-migrate.5.2: "a candidate that adds one finding adds one `+ `
/// line" — check on the hand-written v2 tree prints the warning once, so the
/// comparison lists it once.
#[test]
fn migrate_lists_an_unread_workspace_block_warning_once() {
    let fixture = Fixture::new("migrate");
    let output = grund(&fixture.group(), &["config", "migrate", "--to", "2"]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let compared: Vec<&str> = stderr
        .lines()
        .skip_while(|line| !line.starts_with("differs: grund check --suggestions"))
        .filter(|line| line.starts_with(UNREAD))
        .collect();

    let v2 = Fixture::new("check");
    fs::write(
        v2.group().join("grund.toml"),
        "grund_config_version = 2\n\n[workspace]\ninclude_root = false\nmembers = [\"a\"]\n",
    )
    .unwrap();
    let check = grund(&v2.group(), &["check", "--suggestions"]);
    let printed = String::from_utf8(check.stderr).unwrap();
    let once = printed
        .lines()
        .filter(|line| line.starts_with("warning: ") && line.contains("no project scans"))
        .count();
    assert_eq!(once, 1, "check prints the warning once: {printed}");
    assert_eq!(
        compared.len(),
        once,
        "one finding, one comparison line:\n{stderr}"
    );
}
