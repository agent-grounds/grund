//! §FS-config.3.9.5.2: a narrowed `cover` judges the root's pinned `[citations]`
//! targets against the kinds its declared workspace configures, and judges none
//! where that workspace does not expand. One workspace root pins `api/API`, which
//! its member `api` configures: the pin loads on every command while the member
//! does, and a member config that fails to load never makes the narrowed read's
//! root-only fallback refuse it. A pin the whole reading cannot accept still fails
//! the narrowed load.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const API_CONFIG: &str = "grund_config_version = 1
project_name = \"api\"

[reference]
marker = \"\u{a7}\"
strict = true

[id]
format = \"{kind}-{slug}\"
slug_pattern = \"[a-z][a-z0-9-]*\"

[[kinds]]
kind = \"API\"
folder = \"docs/api\"
index = false

[scan]
include = [\"docs\"]
";

/// A member config the loader refuses: `kind` is not a string.
const BROKEN_API_CONFIG: &str = "grund_config_version = 1
project_name = \"api\"

[[kinds]]
kind = 3
";

struct Workspace(PathBuf);

impl Workspace {
    /// The workspace root, its `[citations.FS]` pinning `target`.
    fn new(target: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-881-narrowed-cover-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let workspace = Self(root);
        workspace.write(
            "grund.toml",
            &format!(
                "grund_config_version = 1
project_name = \"root\"

[reference]
marker = \"\u{a7}\"
strict = true

[id]
format = \"{{kind}}-{{slug}}\"
slug_pattern = \"[a-z][a-z0-9-]*\"

[[kinds]]
kind = \"FS\"
folder = \"docs/fs\"
index = false

[scan]
include = [\"docs\"]

[workspace]
members = [\"api\"]
include_root = true

[citations.FS]
should = [\"{target}\"]
"
            ),
        );
        workspace.write("docs/fs/FS-x.md", "# FS-x: x\n\nText.\n");
        workspace.write("api/grund.toml", API_CONFIG);
        workspace.write("api/docs/api/API-a.md", "# API-a: a\n\nText.\n");
        workspace
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
        fs::write(path, contents).expect("write fixture file");
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.0, args)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run grund")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// While the member loads, the pin it accepts loads on the full, the narrowed
/// `check` and the narrowed `cover` run alike, and none of them says a word about it.
#[test]
fn a_pin_the_member_accepts_loads_on_every_run() {
    let workspace = Workspace::new("api/API");
    for args in [&["check"][..], &["check", "docs"], &["cover", "docs"]] {
        let output = workspace.run(args);
        assert_eq!(output.status.code(), Some(0), "grund {args:?}: {output:?}");
        assert!(
            !stderr(&output).contains("[citations.FS]"),
            "grund {args:?}: {}",
            stderr(&output)
        );
    }
}

/// A member config that fails to load leaves the narrowed `cover` only the root's
/// own kinds, which are not every project's, so it refuses no pinned target on
/// their word; the member's own error stays the full run's to report.
#[test]
fn a_member_that_fails_to_load_never_makes_the_narrowed_cover_refuse_its_pin() {
    let workspace = Workspace::new("api/API");
    workspace.write("api/grund.toml", BROKEN_API_CONFIG);

    let cover = workspace.run(&["cover", "docs"]);
    assert!(
        !stderr(&cover).contains("names an unknown project alias"),
        "{}",
        stderr(&cover)
    );
    assert_eq!(cover.status.code(), Some(0), "{cover:?}");

    for args in [&["check"][..], &["check", "docs"]] {
        let output = workspace.run(args);
        assert_eq!(output.status.code(), Some(2), "grund {args:?}: {output:?}");
        assert!(
            stderr(&output).contains("error: api/grund.toml:"),
            "grund {args:?}: {}",
            stderr(&output)
        );
    }
}

/// The whole declared reading still judges: a pin naming an alias the workspace
/// does not load fails the narrowed `cover` as it fails the full run.
#[test]
fn a_pin_the_whole_reading_cannot_accept_still_fails_the_narrowed_cover() {
    let workspace = Workspace::new("nope/API");
    let expected = "error: grund.toml: [citations.FS] should names an unknown project alias \
                    `nope`; this became an error in grund 0.19.0\n";
    for args in [&["check"][..], &["cover", "docs"]] {
        let output = workspace.run(args);
        assert_eq!(output.status.code(), Some(2), "grund {args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "grund {args:?}: {output:?}");
        assert!(
            stderr(&output).ends_with(expected),
            "grund {args:?}: {}",
            stderr(&output)
        );
    }
}
