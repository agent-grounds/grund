//! A pattern in git's global `core.excludesFile` anchors at the top of the
//! repository its path belongs to, wherever the run starts
//! (§FS-scan.scope.ignore-files.global). Git-backed, because a committed fixture
//! can carry neither a `.git` nor a global git configuration.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 2\n\n\
     [schema]\nid_format = \"{kind}-{slug}\"\n\n\
     [schema.sources]\nrespect_gitignore = true\n\n\
     [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\n\
     [schema.kinds.code]\nfolders = [\"docs\", \"notes\"]\n";

/// What a run reads of [`repository`] once `notes/gen/` is left out.
const LISTED: &str = "FS-x  docs/fs/x.md:1  X\n";

const IGNORED: &str = "# FS-y: Y\n\nCites \u{a7}FS-gone.\n";

/// A fresh fixture directory, `<temp>/grund-global-excludes-<name>-<pid>`, holding
/// a git configuration whose global excludes file is `global.ignore` beside it, with
/// `notes/gen/` its one line. The home directory is the fixture's too, so no
/// `~/.gitconfig` names another excludes file.
fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "grund-global-excludes-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("home")).expect("create fixture home");
    let global = dir.join("global.ignore");
    fs::write(&global, "notes/gen/\n").expect("write global excludes");
    write(
        &dir,
        "xdg/git/config",
        &format!("[core]\nexcludesFile = {}\n", global.display()),
    );
    dir
}

/// `root` as a git repository with the config above: `docs/fs/x.md` declares
/// `FS-x`, `docs/fs/refs.md` cites it, and `notes/gen/b.md`, which the global
/// pattern ignores, declares `FS-y` and cites an ID nothing declares.
fn repository(root: &Path, config: Option<&str>) {
    git_init(root);
    if let Some(config) = config {
        write(root, "grund.toml", config);
    }
    write(root, "docs/fs/x.md", "# FS-x: X\n\nLead.\n");
    write(root, "docs/fs/refs.md", "Keeps \u{a7}FS-x live.\n");
    write(root, "notes/gen/b.md", IGNORED);
}

fn git_init(root: &Path) {
    fs::create_dir_all(root).expect("create repository");
    let git = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(root)
        .output()
        .expect("spawn git init");
    assert!(git.status.success(), "git init {}", root.display());
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
    fs::write(path, body).expect("write fixture");
}

/// `grund` run from `cwd` with the fixture's git configuration and home.
fn grund(fixture: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .env("GIT_CONFIG_GLOBAL", fixture.join("xdg/git/config"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("XDG_CONFIG_HOME", fixture.join("xdg"))
        .env("HOME", fixture.join("home"))
        .env("USERPROFILE", fixture.join("home"))
        .current_dir(cwd)
        .output()
        .expect("run grund")
}

/// Stdout and exit code, the two things each case compares.
fn verdict(output: &Output) -> (String, Option<i32>) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        output.status.code(),
    )
}

fn success() -> (String, Option<i32>) {
    ("success\n".to_string(), Some(0))
}

/// From `docs/` the global file still ignores `notes/gen/`, so its declaration is not
/// listed and its dangling citation is not reported: the run reads what it reads from
/// the top, as git ignores what it ignores from the top.
#[test]
#[ignore = "red until #785 anchors the global excludes at the repository top; implement removes this"]
fn an_anchored_global_pattern_is_not_read_from_a_subdirectory() {
    let dir = fixture("subdirectory");
    let root = dir.join("repo");
    repository(&root, Some(CONFIG));
    let docs = root.join("docs");
    assert_eq!(
        verdict(&grund(&dir, &docs, &["list"])),
        (LISTED.into(), Some(0))
    );
    assert_eq!(verdict(&grund(&dir, &docs, &["check"])), success());
    let _ = fs::remove_dir_all(dir);
}

/// From a directory outside the repository, `check <repo>` anchors there too.
#[test]
#[ignore = "red until #785 anchors the global excludes at the repository top; implement removes this"]
fn an_anchored_global_pattern_is_not_read_from_outside_the_repository() {
    let dir = fixture("outside");
    let root = dir.join("repo");
    repository(&root, Some(CONFIG));
    assert_eq!(verdict(&grund(&dir, &dir, &["check", "repo"])), success());
    let _ = fs::remove_dir_all(dir);
}

/// A config root above the repository top does not move the anchor: run from the
/// config root, `repo/notes/gen/` is ignored, while `notes/gen/` beside the config,
/// in no repository, is matched by no global pattern and is read, as git reads it.
#[test]
#[ignore = "red until #785 anchors the global excludes at the repository top; implement removes this"]
fn an_anchored_global_pattern_anchors_below_a_config_root_above_the_repository() {
    let dir = fixture("config-above");
    let config_root = dir.join("p");
    if let Some(git) = config_root.ancestors().find(|a| a.join(".git").exists()) {
        panic!(
            "this case needs a config root in no git repository, but {} holds `.git`; \
             point TMPDIR outside every git tree",
            git.display()
        );
    }
    repository(&config_root.join("repo"), None);
    write(
        &config_root,
        "grund.toml",
        &CONFIG
            .replace("[\"docs/fs\"]", "[\"repo/docs/fs\"]")
            .replace(
                "[\"docs\", \"notes\"]",
                "[\"notes\", \"repo/docs\", \"repo/notes\"]",
            ),
    );
    write(&config_root, "notes/gen/d.md", "# FS-w: W\n");
    write(&config_root, "notes/refs.md", "Keeps \u{a7}FS-w live.\n");
    assert_eq!(
        verdict(&grund(&dir, &config_root, &["list"])),
        (
            "FS-w  notes/gen/d.md:1  W\nFS-x  repo/docs/fs/x.md:1  X\n".into(),
            Some(0)
        )
    );
    let _ = fs::remove_dir_all(dir);
}

/// A repository nested in the scanned one anchors its own paths at its own top, as
/// its git does, and the outer paths keep the outer top, from either top.
#[test]
#[ignore = "red until #785 anchors the global excludes at the repository top; implement removes this"]
fn a_nested_repository_anchors_its_paths_at_its_own_top() {
    let dir = fixture("nested");
    let root = dir.join("repo");
    repository(&root, Some(CONFIG));
    let inner = root.join("notes/sub");
    git_init(&inner);
    write(&inner, "notes/gen/c.md", "# FS-z: Z\n");
    for cwd in [root.clone(), inner] {
        assert_eq!(
            verdict(&grund(&dir, &cwd, &["list"])),
            (LISTED.into(), Some(0)),
            "from {}",
            cwd.display()
        );
    }
    let _ = fs::remove_dir_all(dir);
}

/// Precedence stays git's: a `.gitignore` re-include outranks the global file, so
/// `notes/gen/b.md` is read from the top and from `docs/` alike.
#[test]
fn a_gitignore_re_include_outranks_the_global_file() {
    let dir = fixture("re-include");
    let root = dir.join("repo");
    repository(&root, Some(CONFIG));
    write(&root, ".gitignore", "!notes/gen/\n");
    let listed = format!("{LISTED}FS-y  notes/gen/b.md:1  Y\n");
    for cwd in [root.clone(), root.join("docs")] {
        assert_eq!(
            verdict(&grund(&dir, &cwd, &["list"])),
            (listed.clone(), Some(0)),
            "from {}",
            cwd.display()
        );
    }
    let _ = fs::remove_dir_all(dir);
}
