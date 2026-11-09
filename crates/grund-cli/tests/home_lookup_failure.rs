//! Black-box regressions for §FS-scan.scope.homes.lookup: a home whose lookup fails for a
//! reason other than absence is a scan failure at the home (§FS-check.2.4), and only a home
//! that does not exist scans as nothing. Directory modes cannot be committed, so the
//! fixtures are built at run time.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

const V2: &str = "grund_config_version = 2\nproject_name = \"fetch-home-lookup-denied\"\n\n\
    [schema.kinds.CONST]\nfolders = [\"data/values\"]\nindex = false\n\
    fetch = \"scripts/fetch-const\"\n";
const V1_SHOULD: &str = "grund_config_version = 1\nproject_name = \"fetch-home-lookup-denied\"\n\n\
    [reference]\nstrict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\
    slug_pattern = \"[a-z][a-z-]*\"\n\n[[kinds]]\nkind = \"CONST\"\n\
    folder = \"data/values\"\nindex = false\nresolve = \"should\"\n\
    fetch = \"scripts/fetch-const\"\n";
const V1_INCLUDE: &str = "grund_config_version = 1\nproject_name = \"include-lookup-denied\"\n\n\
    [scan]\ninclude = [\"docs\", \"data/extra\"]\n\n[id]\nformat = \"{kind}-{slug}\"\n\
    slug_pattern = \"[a-z][a-z-]*\"\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
    index = false\n";
const CITING: &str = "# Offer\n\nThe room holds at most §CONST-room-limit people.\n";
const PLAIN: &str = "# Offer\n\nNothing cited here.\n";
const FIX: &str = "permission denied; make the path readable, or keep it out of the scan";
/// §FS-check.2.4.3: a scan root's fix says how, because exclude does not prune one.
const ROOT_V2: &str = "permission denied; make the path readable, or keep it out of the scan \
    by removing it from the [schema.kinds] row that lists it, because exclude does not prune \
    scan roots";
const ROOT_V1: &str = "permission denied; make the path readable, or keep it out of the scan \
    by removing it from the [scan] include or [[kinds]] row that lists it, because exclude \
    does not prune scan roots";

struct Fixture {
    root: PathBuf,
    locked: Option<PathBuf>,
}

impl Fixture {
    /// A project with `docs/offer.md`; the home is added by the caller's shape.
    fn bare(name: &str, config: &str, doc: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1318-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("grund.toml"), config).unwrap();
        fs::write(root.join("docs/offer.md"), doc).unwrap();
        Self { root, locked: None }
    }

    /// The bare project with the snapshot `docs/offer.md` cites on disk in its home.
    fn with_snapshot(name: &str, config: &str, doc: &str) -> Self {
        let fixture = Self::bare(name, config, doc);
        fs::create_dir_all(fixture.root.join("data/values")).unwrap();
        fs::write(
            fixture.root.join("data/values/CONST-room-limit.md"),
            "# CONST-room-limit: Room limit\n\nRoom limit is 42.\n",
        )
        .unwrap();
        fixture
    }

    /// Sets `data/`'s mode. False when `below` can still be looked up though it should
    /// not be, as for a privileged user, so the caller skips.
    fn lock_data(&mut self, mode: u32, below: &str, lookup_fails: bool) -> bool {
        let data = self.root.join("data");
        fs::set_permissions(&data, fs::Permissions::from_mode(mode)).unwrap();
        self.locked = Some(data);
        let failed = fs::symlink_metadata(self.root.join(below)).is_err();
        failed == lookup_fails && (lookup_fails || fs::read_dir(self.root.join("data")).is_err())
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .current_dir(&self.root)
            .output()
            .expect("run grund")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(path) = &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn all_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        stderr(output)
    )
}

fn lines_equal(text: &str, wanted: &str) -> usize {
    text.lines().filter(|line| *line == wanted).count()
}

fn assert_exit(output: &Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{}", all_text(output));
}

/// §FS-check.2.4.2: the parent the wider walk met and the home are two paths, so two lines.
#[test]
fn v2_denied_lookup_is_a_failure_at_the_home() {
    let mut fixture = Fixture::with_snapshot("v2", V2, CITING);
    if !fixture.lock_data(0o000, "data/values", true) {
        return;
    }
    let output = fixture.check();
    let err = stderr(&output);
    assert_eq!(
        lines_equal(&err, &format!("error: data/values: {ROOT_V2}")),
        1,
        "{err}"
    );
    assert_eq!(
        lines_equal(&err, &format!("error: data: {FIX}")),
        1,
        "{err}"
    );
    assert_exit(&output, 2);
}

/// §FS-check.2.4: no wider root meets `data` here, so the home's line is the only report.
#[test]
fn v1_should_denied_lookup_is_a_failure_at_the_home_and_exits_2() {
    let mut fixture = Fixture::with_snapshot("v1", V1_SHOULD, CITING);
    if !fixture.lock_data(0o000, "data/values", true) {
        return;
    }
    let output = fixture.check();
    let err = stderr(&output);
    assert_eq!(
        lines_equal(&err, &format!("error: data/values: {ROOT_V1}")),
        1,
        "{err}"
    );
    assert_exit(&output, 2);
}

/// §FS-scan.scope.homes.lookup: an unread home earns no statement about what it holds. The
/// failure is recorded at the home, so §FS-check.2.4 withholds the citation's finding.
#[test]
fn denied_lookup_never_says_no_snapshot() {
    for (name, config) in [("snap-v2", V2), ("snap-v1", V1_SHOULD)] {
        let mut fixture = Fixture::with_snapshot(name, config, CITING);
        if !fixture.lock_data(0o000, "data/values", true) {
            return;
        }
        let text = all_text(&fixture.check());
        assert!(!text.contains("no snapshot"), "{name}: {text}");
    }
}

/// §FS-scan.scope.homes.lookup: the rule is the walk's for every root.
#[test]
fn denied_lookup_of_an_include_entry_is_a_failure_at_the_entry() {
    let mut fixture = Fixture::bare("include", V1_INCLUDE, "# FS-a: A\n\nPlain.\n");
    fs::create_dir_all(fixture.root.join("data/extra")).unwrap();
    if !fixture.lock_data(0o000, "data/extra", true) {
        return;
    }
    let output = fixture.check();
    let err = stderr(&output);
    assert_eq!(
        lines_equal(&err, &format!("error: data/extra: {ROOT_V1}")),
        1,
        "{err}"
    );
    assert_exit(&output, 2);
}

/// §FS-scan.scope.homes.lookup: a parent that cannot be listed does not leave the home unread.
#[test]
fn unlisted_parent_is_reported_alone_and_the_home_is_read() {
    let mut fixture = Fixture::with_snapshot("parent", V2, CITING);
    if !fixture.lock_data(0o300, "data/values", false) {
        return;
    }
    let output = fixture.check();
    let text = all_text(&output);
    assert_eq!(
        lines_equal(&stderr(&output), &format!("error: data: {FIX}")),
        1,
        "{text}"
    );
    assert!(!text.contains("no snapshot"), "{text}");
    assert!(!text.contains("unknown reference"), "{text}");
    assert!(!text.contains("data/values"), "{text}");
    assert_exit(&output, 2);
}

/// §FS-scan.scope.homes: a home that does not exist scans as nothing.
#[test]
fn absent_home_scans_as_nothing() {
    let output = Fixture::bare("absent", V2, PLAIN).check();
    assert!(
        !all_text(&output).contains("error:"),
        "{}",
        all_text(&output)
    );
    assert_exit(&output, 0);
}

/// §FS-scan.scope.homes.lookup: below a regular file nothing can exist, so this is absence.
#[test]
fn home_below_a_regular_file_scans_as_nothing() {
    let fixture = Fixture::bare("below-file", V2, PLAIN);
    fs::write(fixture.root.join("data"), "not a directory\n").unwrap();
    let output = fixture.check();
    assert!(
        !all_text(&output).contains("error:"),
        "{}",
        all_text(&output)
    );
    assert_exit(&output, 0);
}

/// §FS-scan.scope.homes.lookup: a name the file system cannot hold names nothing, so this is
/// absence; a component over the length limit is the Unix form of a `*` on Windows.
#[test]
fn root_whose_name_cannot_be_held_scans_as_nothing() {
    let config = V1_INCLUDE.replace("data/extra", &"x".repeat(300));
    let output = Fixture::bare("name-too-long", &config, "# FS-a: A\n\nPlain.\n").check();
    assert!(
        !all_text(&output).contains("error:"),
        "{}",
        all_text(&output)
    );
    assert_exit(&output, 0);
}

/// A tree beside the project, whose `par/t` the outward link names, with `par` denied.
fn outside_tree(fixture: &Fixture, name: &str) -> PathBuf {
    let outside = fixture
        .root
        .with_file_name(format!("grund-1318-{name}-outside-{}", std::process::id()));
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(outside.join("par/t")).unwrap();
    outside
}

const V1_DEFAULT_ROOTS: &str = "grund_config_version = 1\nproject_name = \"outward-root\"\n\n\
    [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n";
const V2_OUTWARD_HOME: &str = "grund_config_version = 2\nproject_name = \"outward-root\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\n\
    [schema.kinds.CONST]\nfolders = [\"vals\"]\nindex = false\n";

/// Runs `check` with `link` naming `par/t` outside the project and `par` denied.
fn outward_denied_check(name: &str, config: &str, link: &str) -> Option<Output> {
    let fixture = Fixture::bare(name, config, "# FS-a: A\n\nPlain.\n");
    let outside = outside_tree(&fixture, name);
    std::os::unix::fs::symlink(outside.join("par/t"), fixture.root.join(link)).unwrap();
    let par = outside.join("par");
    fs::set_permissions(&par, fs::Permissions::from_mode(0o000)).unwrap();
    let denied = fs::symlink_metadata(par.join("t")).is_err();
    let output = denied.then(|| fixture.check());
    fs::set_permissions(&par, fs::Permissions::from_mode(0o755)).unwrap();
    let _ = fs::remove_dir_all(&outside);
    output
}

/// §FS-scan.scope.homes.lookup.outward: pruned as it is with the target open, so a mode
/// outside the project decides nothing.
#[test]
fn outward_root_link_with_a_denied_parent_is_pruned_on_version_1() {
    let Some(output) = outward_denied_check("out-v1", V1_DEFAULT_ROOTS, "src") else {
        return;
    };
    assert!(
        !stderr(&output).contains("permission denied"),
        "{}",
        stderr(&output)
    );
    assert_exit(&output, 0);
}

/// §FS-scan.scope.homes.lookup.outward: a kind home that is such a link is the same root.
#[test]
fn outward_home_link_with_a_denied_parent_is_pruned_on_version_2() {
    let Some(output) = outward_denied_check("out-v2", V2_OUTWARD_HOME, "vals") else {
        return;
    };
    assert!(
        !stderr(&output).contains("permission denied"),
        "{}",
        stderr(&output)
    );
    assert_exit(&output, 0);
}

/// §FS-scan.scope.homes.lookup.outward: a file link is followed wherever it resolves, so a
/// root named as a file the walk reads stays reported.
#[test]
fn outward_root_link_named_as_a_read_file_is_reported() {
    const INCLUDE: &str = "grund_config_version = 1\nproject_name = \"outward-root\"\n\n\
        [scan]\ninclude = [\"docs\", \"far.md\"]\n\n\
        [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
        [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n";
    let Some(output) = outward_denied_check("out-file", INCLUDE, "far.md") else {
        return;
    };
    assert_eq!(
        lines_equal(&stderr(&output), &format!("error: far.md: {ROOT_V1}")),
        1,
        "{}",
        stderr(&output)
    );
    assert_exit(&output, 2);
}

/// §FS-scan.scope.homes.lookup.outward: a link staying inside the project is still reported.
#[test]
fn inward_root_link_with_a_denied_parent_is_reported() {
    let mut fixture = Fixture::bare("inward", V1_DEFAULT_ROOTS, "# FS-a: A\n\nPlain.\n");
    fs::create_dir_all(fixture.root.join("data/real")).unwrap();
    std::os::unix::fs::symlink("data/real", fixture.root.join("src")).unwrap();
    if !fixture.lock_data(0o000, "data/real", true) {
        return;
    }
    let output = fixture.check();
    assert_eq!(
        lines_equal(&stderr(&output), &format!("error: src: {ROOT_V1}")),
        1,
        "{}",
        stderr(&output)
    );
    assert_exit(&output, 2);
}
