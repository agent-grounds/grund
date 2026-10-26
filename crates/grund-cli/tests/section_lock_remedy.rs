//! §FS-section-lock.checks.remedy: the two lock errors `check` clears with
//! `fmt --write` name the run that writes a lock, scope and all, because the
//! same command given a file leaves every lock alone (§FS-section-lock.write.rules),
//! and a member's own run leaves its entries for a sibling alone
//! (§FS-section-lock.file.members). The fixture is grund#850's: a mixed address
//! cited from a source file, and no `.grund/` yet.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// §FS-section-lock.checks.remedy: the remedy, worded once.
const REMEDY: &str = "run grund fmt --write with no path at the workspace root, \
                      or at the project root outside a workspace";
const LOCK: &str = ".grund/sections/AR-001-lock.lock";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"lock-remedy\"\n\n\
                      [schema]\nid_format = \"{kind}-{number}-{slug}\"\n\n\
                      [schema.kinds.AR]\nfolders = [\"docs/architecture\"]\nindex = false\n";
const DECLARATION: &str = "# AR-001-lock: Lock\n\n## 1. Overview\n\nOverview text.\n\n### 1.child: Child\n\nChild text.\n";
const SOURCE: &str = "// \u{a7}AR-001-lock.1.child\npub fn f() {}\n";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn empty(what: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-850-{what}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the fixture root");
        Self { root }
    }

    /// The issue's project: `1.child` is a name under a number, so a locked address.
    fn standalone() -> Self {
        let repo = Self::empty("lock-remedy");
        repo.write("grund.toml", CONFIG);
        repo.write("docs/architecture/AR-001-lock.md", DECLARATION);
        repo.write("src/lib.rs", SOURCE);
        repo
    }

    fn write(&self, path: &str, body: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn has(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    /// Runs grund in `cwd`, relative to the fixture root, and returns its stdout
    /// after asserting the exit code.
    fn run(&self, cwd: &str, args: &[&str], code: i32) -> String {
        let output: Output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.root.join(cwd))
            .output()
            .expect("spawn grund");
        let (stdout, stderr) = (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        );
        assert_eq!(
            output.status.code(),
            Some(code),
            "grund {args:?} in {cwd:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        stdout
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unrecorded(lock: &str) -> String {
    format!(
        "src/lib.rs:1: error: AR-001-lock.1.child is cited but has no entry in {lock}; {REMEDY}\n"
    )
}

/// §FS-section-lock.checks.unrecorded-section-citation: grund#850's `check`.
#[test]
fn an_unrecorded_citation_names_the_root_scoped_run() {
    let repo = Repo::standalone();
    assert_eq!(repo.run("", &["check"], 1), unrecorded(LOCK));
    // The report names the lock from the root wherever `check` ran, and so does the remedy.
    assert_eq!(repo.run("src", &["check"], 1), unrecorded(LOCK));
}

/// §FS-section-lock.checks.invalid-section-lock: the same remedy, worded once.
#[test]
fn an_unreadable_lock_line_names_the_root_scoped_run() {
    let repo = Repo::standalone();
    repo.run("", &["fmt", "--write"], 0);
    let entry = fs::read_to_string(repo.root.join(LOCK)).unwrap();
    repo.write(LOCK, &format!("{entry}not an entry\n"));
    assert_eq!(
        repo.run("", &["check"], 1),
        format!(
            "{LOCK}:2: error: section lock line is not <coordinate> <fingerprint>; \
             to rebuild the file, {REMEDY}\n"
        )
    );
}

/// §FS-section-lock.write.rules: each narrowed run grund#850's agents tried writes
/// nothing, `check` still names the root-scoped run, and that run, as written, writes it.
#[test]
fn a_narrowed_fmt_leaves_the_lock_and_the_named_run_writes_it() {
    let repo = Repo::standalone();
    let narrowed: [(&str, &[&str]); 3] = [
        ("", &["fmt", "src/lib.rs", "--write"]),
        ("", &["fmt", "src", "--write"]),
        ("src", &["fmt", ".", "--write"]),
    ];
    for (cwd, args) in narrowed {
        assert_eq!(
            repo.run(cwd, args, 0),
            "rewrote 0 lines\n",
            "{args:?} in {cwd:?}"
        );
        assert!(!repo.has(LOCK), "{args:?} in {cwd:?} wrote the lock");
        assert_eq!(repo.run("", &["check"], 1), unrecorded(LOCK));
    }
    assert_eq!(
        repo.run("", &["fmt", "--write"], 0),
        format!("rewrote 1 line: 1 section-lock entry\n  {LOCK} (1) created\n")
    );
    assert_eq!(repo.run("", &["check"], 0), "success\n");
}

/// §FS-fmt.7.1, a control: with no path the scope is the project's from anywhere
/// in it, so a bare run from a subdirectory is the run the remedy names.
#[test]
fn a_bare_fmt_from_a_subdirectory_is_the_root_scoped_run() {
    let repo = Repo::standalone();
    assert_eq!(
        repo.run("src", &["fmt", "--write"], 0),
        format!("rewrote 1 line: 1 section-lock entry\n  {LOCK} (1) created\n")
    );
    assert_eq!(repo.run("", &["check"], 0), "success\n");
}

/// §FS-section-lock.file.members: from the workspace root the remedy names the
/// root-scoped run for the root's lock of a member alias and for a member's own lock,
/// and one run there writes both.
#[test]
fn a_member_lock_names_the_same_run_and_the_workspace_root_writes_it() {
    let repo = Repo::empty("lock-remedy-workspace");
    let case = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-section-lock-member-unrecorded/repo");
    copy_tree(&case, &repo.root);
    let (alias, member) = (
        ".grund/sections/api/FS-login.lock",
        "packages/api/.grund/sections/FS-login.lock",
    );
    let report = format!(
        "docs/use.md:3: error: api/FS-login.format.1 is cited but has no entry in {alias}; \
         {REMEDY}\n\
         packages/api/docs/notes.md:3: error: FS-login.format.2 is cited but has no entry in \
         {member}; {REMEDY}\n"
    );
    assert_eq!(repo.run("", &["check"], 1), report);
    assert_eq!(
        repo.run("", &["fmt", "packages/api/docs", "--write"], 0),
        "rewrote 0 lines\n"
    );
    assert!(!repo.has(alias) && !repo.has(member));
    assert_eq!(
        repo.run("", &["fmt", "--write"], 0),
        format!(
            "rewrote 2 lines: 2 section-lock entries\n  {alias} (1) created\n  {member} (1) created\n"
        )
    );
    // The fixture's version 1 configs earn warnings, so a clean run prints no `success`.
    repo.run("", &["check"], 0);
}

/// §FS-section-lock.file.members: a member's entry for a citation into a sibling is
/// written by the workspace root's run alone, so §FS-section-lock.checks.remedy names
/// that root: the member's own run, from inside it or given its root, writes its own
/// lock and leaves that entry missing.
#[test]
fn a_member_lock_of_a_sibling_citation_is_written_only_at_the_workspace_root() {
    let repo = Repo::empty("lock-remedy-sibling");
    let case = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-section-lock-member-unrecorded/repo");
    let declaration = "packages/api/docs/functional-spec/FS-login.md";
    repo.write(
        declaration,
        &fs::read_to_string(case.join(declaration)).unwrap(),
    );
    // Version 2 configs, so a clean run prints `success`; sources, so `fmt` adds no link.
    repo.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"root\"\n\n\
         [workspace]\nmembers = [\"packages/api\", \"packages/web\"]\n",
    );
    repo.write(
        "packages/api/grund.toml",
        "grund_config_version = 2\nproject_name = \"api\"\n\n\
         [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n",
    );
    repo.write(
        "packages/web/grund.toml",
        "grund_config_version = 2\nproject_name = \"web\"\n",
    );
    repo.write(
        "src/lib.rs",
        "// \u{a7}api/FS-login.format.1\npub fn f() {}\n",
    );
    repo.write(
        "packages/api/src/lib.rs",
        "// \u{a7}FS-login.format.2\npub fn f() {}\n",
    );
    repo.write(
        "packages/web/src/lib.rs",
        "// \u{a7}api/FS-login.format.1\npub fn f() {}\n",
    );
    let (root, member, sibling) = (
        ".grund/sections/api/FS-login.lock",
        "packages/api/.grund/sections/FS-login.lock",
        "packages/web/.grund/sections/api/FS-login.lock",
    );
    let report = |locks: &[(&str, &str, &str)]| -> String {
        locks
            .iter()
            .map(|(site, coordinate, lock)| {
                format!(
                    "{site}:1: error: {coordinate} is cited but has no entry in {lock}; \
                     {REMEDY}\n"
                )
            })
            .collect()
    };
    let web = ("packages/web/src/lib.rs", "api/FS-login.format.1", sibling);
    let at_root = ("src/lib.rs", "api/FS-login.format.1", root);
    assert_eq!(
        repo.run("", &["check"], 1),
        report(&[
            ("packages/api/src/lib.rs", "FS-login.format.2", member),
            web,
            at_root,
        ])
    );
    // The member's own run, given its root or from inside it, writes nothing for the sibling.
    let members: [(&str, &[&str]); 2] = [
        ("", &["fmt", "packages/web", "--write"]),
        ("packages/web", &["fmt", "--write"]),
    ];
    for (cwd, args) in members {
        assert_eq!(
            repo.run(cwd, args, 0),
            "rewrote 0 lines\n",
            "{args:?} in {cwd:?}"
        );
        assert!(!repo.has(sibling), "{args:?} in {cwd:?} wrote {sibling}");
    }
    // A member run writes the member's own lock, and still not the sibling entry.
    assert_eq!(
        repo.run("", &["fmt", "packages/api", "--write"], 0),
        "rewrote 1 line: 1 section-lock entry\n  .grund/sections/FS-login.lock (1) created\n"
    );
    assert!(repo.has(member) && !repo.has(sibling));
    assert_eq!(repo.run("", &["check"], 1), report(&[web, at_root]));
    // The run the remedy names writes every entry left.
    assert_eq!(
        repo.run("", &["fmt", "--write"], 0),
        format!(
            "rewrote 2 lines: 2 section-lock entries\n  {root} (1) created\n  {sibling} (1) created\n"
        )
    );
    assert_eq!(repo.run("", &["check"], 0), "success\n");
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).expect("read the e2e fixture") {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
