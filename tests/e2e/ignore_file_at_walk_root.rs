//! An ignore file the walk cannot parse is one failure of the run, whichever walk root met it
//! (§FS-check.2.4, §FS-check.2.4.2): a project walked only from the directory that holds the file
//! reports it and exits `2` exactly as one whose kind home sits below the file does, unless the
//! run does not apply the file (§FS-scan.scope.ignore-files.unparsed). The trees are built in the
//! temp directory, because a malformed `.ignore` committed to this repository would be read by
//! every other tool that honours ignore files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REMEDY: &str = "make the path readable, or keep it out of the scan";
const ALPHA: &str = "error parsing glob '{alpha': unclosed alternate group; \
                     missing '}' (maybe escape '{' with '[{]'?)";

/// One project tree under the temp directory, removed on drop.
struct Tree(PathBuf);

impl Tree {
    /// A version-2 project `p` holding `.ignore` with `ignore` at its root. With `kind_home`, an
    /// `FS` kind keeps its home at `docs/`, so the walk starts below the file; without, the
    /// project declares no kinds and the walk starts at the config root, where the file is.
    fn new(name: &str, kind_home: bool, ignore: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1305-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        let mut config = String::from("grund_config_version = 2\nproject_name = \"p\"\n");
        if kind_home {
            config.push_str(
                "\n[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\n\
                 folders = [\"docs\"]\n",
            );
            fs::create_dir(root.join("docs")).expect("create the kind home");
            fs::write(root.join("docs/x.md"), "# Notes\n").expect("write the note");
        } else {
            fs::write(root.join("x.md"), "# Notes\n").expect("write the note");
        }
        fs::write(root.join("grund.toml"), config).expect("write the config");
        fs::write(root.join(".ignore"), ignore).expect("write the ignore file");
        Self(root)
    }

    /// Writes `content` at `relative` below the project, its directories made as needed.
    fn write(&self, relative: &str, content: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("a path below the tree")).expect("create the dir");
        fs::write(path, content).expect("write the file");
    }

    fn check(&self) -> Run {
        self.check_with(&[])
    }

    fn check_with(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            root: self.0.clone(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
    root: PathBuf,
}

impl Run {
    fn shown(&self) -> String {
        format!(
            "exit {}\nstdout:\n{}\nstderr:\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    /// The run's own failure lines (§FS-errors.2.2): a located finding reads `path:line:`.
    fn failure_lines(&self) -> Vec<&str> {
        self.stderr
            .lines()
            .filter(|line| line.starts_with("error: "))
            .collect()
    }

    fn mentions_root(&self, root: &Path) -> bool {
        let root = root.to_string_lossy();
        self.stdout.contains(root.as_ref()) || self.stderr.contains(root.as_ref())
    }
}

/// The one line a single malformed line in `.ignore` earns, wherever the walk met it.
fn one_line_failure() -> String {
    format!("error: .ignore: line 1: {ALPHA}; {REMEDY}")
}

#[test]
fn a_malformed_ignore_file_at_the_only_walk_root_is_reported_once() {
    let tree = Tree::new("root-one", false, "{alpha\n");
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(run.stdout, "", "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!("{}\n", one_line_failure()),
        "{}",
        run.shown()
    );
}

#[test]
fn two_malformed_lines_at_the_only_walk_root_are_reported_at_the_ignore_file() {
    // Two bad lines reach the walk as one `ignore::Error::Partial`, which names no path itself.
    let tree = Tree::new("root-two", false, "{alpha\n{beta\n");
    assert_two_lines_are_one_failure(&tree.check());
}

/// Asserts the run exits `2` with exactly one failure line at `.ignore`, the ignore file with
/// two bad lines, which names both globs (§FS-check.2.4.2) and no absolute path (§FS-errors.4).
fn assert_two_lines_are_one_failure(run: &Run) {
    assert_eq!(run.code, 2, "{}", run.shown());
    let lines = run.failure_lines();
    assert_eq!(lines.len(), 1, "not one line:\n{}", run.shown());
    assert!(lines[0].starts_with("error: .ignore:"), "{}", run.shown());
    for glob in ["'{alpha'", "'{beta'"] {
        assert!(
            run.stderr.contains(glob),
            "{glob} unnamed:\n{}",
            run.shown()
        );
    }
    assert!(
        !run.mentions_root(&run.root),
        "absolute path:\n{}",
        run.shown()
    );
}

#[test]
fn a_malformed_ignore_file_above_a_kind_home_is_reported_once() {
    let tree = Tree::new("kind-home", true, "{alpha\n");
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(run.stdout, "", "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!("{}\n", one_line_failure()),
        "{}",
        run.shown()
    );
}

/// Asserts the run exits `2` printing exactly the one failure line for `{alpha` at `path`.
fn assert_one_failure_at(run: &Run, path: &str) {
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(
        run.failure_lines(),
        [format!("error: {path}: line 1: {ALPHA}; {REMEDY}")],
        "{}",
        run.shown()
    );
}

#[test]
fn a_malformed_ignore_file_in_a_subdirectory_is_reported_at_its_own_path() {
    let tree = Tree::new("nested", false, "");
    tree.write("sub/y.md", "# Notes\n");
    tree.write("sub/.ignore", "{alpha\n");
    assert_one_failure_at(&tree.check(), "sub/.ignore");
}

#[test]
fn a_malformed_ignore_file_above_a_second_walk_root_is_reported_once() {
    // `--full` walks the kind home beside the config root: one walk starts at the file's
    // directory, the other below it.
    let tree = Tree::new("two-walks", true, "{alpha\n");
    assert_one_failure_at(&tree.check_with(&["--full"]), ".ignore");
}

#[test]
fn a_malformed_ignore_file_a_kind_home_holds_is_reported_once() {
    // The config-root walk descends to it; the kind-home walk starts in its directory.
    let tree = Tree::new("home-own", true, "");
    tree.write("docs/.ignore", "{alpha\n");
    assert_one_failure_at(&tree.check_with(&["--full"]), "docs/.ignore");
}

#[cfg(unix)]
#[test]
fn a_malformed_ignore_file_reached_through_a_link_is_reported() {
    let tree = Tree::new("linked", false, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    tree.write("rules.txt", "{alpha\n");
    std::os::unix::fs::symlink("rules.txt", tree.0.join(".ignore")).expect("link the file");
    assert_one_failure_at(&tree.check(), ".ignore");
}

#[cfg(unix)]
#[test]
fn two_malformed_lines_in_a_linked_ignore_file_are_one_failure() {
    let tree = Tree::new("linked-two", false, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    tree.write("rules.txt", "{alpha\n{beta\n");
    std::os::unix::fs::symlink("rules.txt", tree.0.join(".ignore")).expect("link the file");
    assert_two_lines_are_one_failure(&tree.check());
    // The same file above a kind home meets the walk by the other route.
    let home = Tree::new("linked-two-home", true, "");
    fs::remove_file(home.0.join(".ignore")).expect("clear the ignore file");
    home.write("rules.txt", "{alpha\n{beta\n");
    std::os::unix::fs::symlink("rules.txt", home.0.join(".ignore")).expect("link the file");
    assert_two_lines_are_one_failure(&home.check());
}

/// Asserts the run reports no failure and exits `0`.
fn assert_clean(run: &Run) {
    assert_eq!(run.code, 0, "{}", run.shown());
    assert_eq!(run.failure_lines(), Vec::<&str>::new(), "{}", run.shown());
}

/// A kind-less project whose `[schema.sources] exclude` holds `pattern`.
fn excluding(name: &str, pattern: &str) -> Tree {
    let tree = Tree::new(name, false, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    let config = format!(
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.sources]\n\
         exclude = [\"{pattern}\"]\n"
    );
    tree.write("grund.toml", &config);
    tree
}

/// `excluding`, with an `FS` kind whose home is `home`, so a walk starts there.
fn excluding_with_home(name: &str, pattern: &str, home: &str) -> Tree {
    let tree = excluding(name, pattern);
    let config = format!(
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.sources]\nexclude = [\"{pattern}\"]\n\n\
         [schema.kinds.FS]\nfolders = [\"{home}\"]\n"
    );
    tree.write("grund.toml", &config);
    tree.write(&format!("{home}/x.md"), "# N\n");
    tree
}

#[test]
fn a_malformed_ignore_file_exclude_keeps_out_is_no_failure() {
    for pattern in ["vendor/**", "vendor/*"] {
        let tree = excluding("excluded", pattern);
        tree.write("vendor/.ignore", "{alpha\n");
        assert_clean(&tree.check());
    }
}

#[cfg(unix)]
#[test]
fn exclude_treats_an_unparsable_ignore_file_as_it_treats_an_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;
    let tree = excluding("excluded-control", "vendor/**");
    tree.write("vendor/a.md", "# A\n");
    let barred = tree.0.join("vendor/a.md");
    fs::set_permissions(&barred, fs::Permissions::from_mode(0o000)).expect("bar the file");
    if fs::read(&barred).is_ok() {
        eprintln!("skipped: this user reads through the mode");
        return;
    }
    assert_clean(&tree.check());
    tree.write("vendor/.ignore", "{alpha\n");
    assert_clean(&tree.check());
    fs::set_permissions(&barred, fs::Permissions::from_mode(0o644)).expect("restore the file");
}

#[test]
fn excluding_the_ignore_file_by_name_leaves_its_rules_in_force_so_the_line_still_fails() {
    // §FS-scan.scope.ignore-files: `exclude` applies beside the file's rules, never instead.
    for (at, pattern) in [".ignore", "**/.ignore", ".*"].into_iter().enumerate() {
        let tree = excluding(&format!("excluded-name-{at}"), pattern);
        tree.write(".ignore", "{alpha\n");
        assert_one_failure_at(&tree.check(), ".ignore");
        // Asked by the other route too: the walk below the file meets it from above.
        let home = excluding_with_home(&format!("excluded-name-home-{at}"), pattern, "docs");
        home.write(".ignore", "{alpha\n");
        assert_one_failure_at(&home.check(), ".ignore");
    }
}

#[test]
fn a_pattern_that_keeps_out_only_part_of_the_directory_leaves_the_line_failing() {
    let some = excluding("excluded-some", "vendor/*.md");
    some.write("vendor/.ignore", "{alpha\n");
    assert_one_failure_at(&some.check(), "vendor/.ignore");
    // A negation may rescue any name, so what the directory holds is not all kept out.
    let negated = excluding("excluded-negated", "vendor/**");
    negated.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.sources]\n\
         exclude = [\"vendor/**\", \"!vendor/keep.md\"]\n",
    );
    negated.write("vendor/.ignore", "{alpha\n");
    assert_one_failure_at(&negated.check(), "vendor/.ignore");
}

#[test]
fn a_malformed_ignore_file_in_an_excluded_kind_home_is_no_failure() {
    let config = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
                  slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.sources]\n\
                  exclude = [\"docs/**\"]\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n";
    let tree = Tree::new("excluded-home", true, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    tree.write("grund.toml", config);
    tree.write("docs/.ignore", "{alpha\n");
    assert_eq!(tree.check().failure_lines(), Vec::<&str>::new());
}

#[test]
fn a_kind_home_below_an_excluded_directory_is_read_under_that_directorys_ignore_file() {
    // The answers differ because what is read differs: the walk from the config root prunes
    // `vendor/` and applies nothing there, and the walk from the kind home applies the file.
    let pruned = excluding("excluded-dir", "vendor/");
    pruned.write("vendor/sub/x.md", "# N\n");
    pruned.write("vendor/.ignore", "{alpha\n");
    assert_clean(&pruned.check());
    let read = excluding_with_home("excluded-above-home", "vendor/", "vendor/sub");
    read.write("vendor/.ignore", "{alpha\n");
    assert_one_failure_at(&read.check(), "vendor/.ignore");
}

#[test]
fn one_walk_that_reads_under_the_ignore_file_is_enough_to_report_it_once() {
    // `vendor/*` keeps out all the config-root walk finds in `vendor/`, and nothing the walk from
    // the kind home two levels down reads.
    let tree = excluding_with_home("one-walk-applies", "vendor/*", "vendor/sub/deep");
    tree.write("vendor/.ignore", "{alpha\n");
    assert_one_failure_at(&tree.check(), "vendor/.ignore");
    assert_one_failure_at(&tree.check_with(&["--full"]), "vendor/.ignore");
}

#[test]
fn a_malformed_ignore_file_the_exclude_does_not_reach_is_still_reported() {
    let tree = excluding("not-excluded", "vendor/**");
    tree.write("docs/.ignore", "{alpha\n");
    assert_one_failure_at(&tree.check(), "docs/.ignore");
}

/// A project with a malformed `.gitignore` at its root, its kind home at `docs/` when `kind_home`.
fn with_gitignore(name: &str, kind_home: bool) -> Tree {
    let tree = Tree::new(name, kind_home, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    tree.write(".gitignore", "{alpha\nx.md\n");
    tree
}

/// Whether a repository encloses the temp directory, so no tree built there is outside one. The
/// test that asks prints why it stopped, as the unreadable-file test does under root.
fn temp_dir_in_a_repository() -> bool {
    let temp = std::env::temp_dir();
    let temp = temp.canonicalize().unwrap_or(temp);
    let enclosed = temp
        .ancestors()
        .any(|dir| dir.join(".git").exists() || dir.join(".jj").exists());
    if enclosed {
        eprintln!("skipped: a repository encloses the temp directory");
    }
    enclosed
}

#[test]
fn a_gitignore_outside_any_repository_is_no_failure() {
    if temp_dir_in_a_repository() {
        return;
    }
    // The walk would not apply the file's rules even had it parsed: `x.md` is read.
    let tree = with_gitignore("gitignore-no-repo", false);
    assert_clean(&tree.check());
    // Nor by the route a kind home below takes, which is the same answer.
    let home = with_gitignore("gitignore-no-repo-home", true);
    assert_clean(&home.check());
    let nested = with_gitignore("gitignore-no-repo-nested", false);
    fs::remove_file(nested.0.join(".gitignore")).expect("clear the root file");
    nested.write("sub/.gitignore", "{alpha\n");
    nested.write("sub/y.md", "# N\n");
    assert_clean(&nested.check());
}

#[test]
fn a_gitignore_inside_a_repository_is_reported() {
    let tree = with_gitignore("gitignore-repo", false);
    fs::create_dir(tree.0.join(".git")).expect("create the repository marker");
    assert_one_failure_at(&tree.check(), ".gitignore");
    let jj = with_gitignore("gitignore-jj", false);
    fs::create_dir(jj.0.join(".jj")).expect("create the repository marker");
    assert_one_failure_at(&jj.check(), ".gitignore");
    let nested = with_gitignore("gitignore-repo-nested", false);
    fs::create_dir(nested.0.join(".git")).expect("create the repository marker");
    fs::remove_file(nested.0.join(".gitignore")).expect("clear the root file");
    nested.write("sub/.gitignore", "{alpha\n");
    assert_one_failure_at(&nested.check(), "sub/.gitignore");
}

#[test]
fn a_malformed_dot_ignore_is_reported_with_or_without_a_repository() {
    let bare = Tree::new("dot-ignore-bare", false, "{alpha\n");
    assert_one_failure_at(&bare.check(), ".ignore");
    let repo = Tree::new("dot-ignore-repo", false, "{alpha\n");
    fs::create_dir(repo.0.join(".git")).expect("create the repository marker");
    assert_one_failure_at(&repo.check(), ".ignore");
}

#[test]
fn a_repositorys_malformed_info_exclude_is_reported_under_the_default_name_list() {
    // `.git` is on version 1's default name list, and the file governs the repository whatever
    // that list says of the directory it sits in.
    for (name, kind_home, kinds) in [
        (
            "info-exclude",
            false,
            "[scan]\ninclude = [\".\"]\n\n[[kinds]]\nkind = \"FS\"\nfolder = \".\"\n",
        ),
        (
            "info-exclude-home",
            true,
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
        ),
    ] {
        let tree = Tree::new(name, kind_home, "");
        fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
        tree.write(
            "grund.toml",
            &format!("grund_config_version = 1\n\n{kinds}"),
        );
        tree.write(".git/info/exclude", "{alpha\n");
        assert_one_failure_at(&tree.check(), ".git/info/exclude");
    }
}

/// A version-1 project whose `E2E` folder is a link out of the project, so no walk lists its cases
/// and the manifest pass walks that root itself (§FS-check.2.4.2).
#[cfg(unix)]
fn with_linked_out_cases(name: &str) -> (Tree, PathBuf) {
    let tree = Tree::new(name, false, "");
    fs::remove_file(tree.0.join(".ignore")).expect("clear the ignore file");
    tree.write(
        "grund.toml",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"E2E\"\nfolder = \"tests/e2e\"\n",
    );
    fs::create_dir(tree.0.join("tests")).expect("create tests");
    let outside = tree.0.with_extension("cases");
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(outside.join("1-alpha")).expect("create the case");
    fs::write(outside.join("1-alpha/expected.exit"), "0\n").expect("write the manifest");
    std::os::unix::fs::symlink(&outside, tree.0.join("tests/e2e")).expect("link the cases");
    (tree, outside)
}

#[cfg(unix)]
#[test]
fn a_malformed_ignore_file_at_a_cases_root_no_walk_lists_is_reported_once() {
    let (tree, outside) = with_linked_out_cases("cases-outside");
    fs::write(outside.join(".ignore"), "{alpha\n").expect("write the ignore file");
    let run = tree.check();
    let _ = fs::remove_dir_all(&outside);
    assert_one_failure_at(&run, "tests/e2e/.ignore");
}

#[cfg(unix)]
#[test]
fn a_gitignore_at_a_cases_root_no_repository_encloses_is_no_failure() {
    if temp_dir_in_a_repository() {
        return;
    }
    // No walk lists this root either, and the pass's report asks the same question the walk's does.
    let (tree, outside) = with_linked_out_cases("cases-outside-gitignore");
    fs::write(outside.join(".gitignore"), "{alpha\n").expect("write the ignore file");
    let run = tree.check();
    let _ = fs::remove_dir_all(&outside);
    assert_clean(&run);
}

#[cfg(unix)]
#[test]
fn a_gitignore_at_a_linked_out_cases_root_inside_a_repository_is_reported() {
    // The repository is met on the path the link resolves to, as the walker finds it.
    let (tree, outside) = with_linked_out_cases("cases-outside-repository");
    fs::create_dir(outside.join(".git")).expect("create the repository marker");
    fs::write(outside.join(".gitignore"), "{alpha\n").expect("write the ignore file");
    let run = tree.check();
    let _ = fs::remove_dir_all(&outside);
    assert_one_failure_at(&run, "tests/e2e/.gitignore");
}

/// Asserts the run exits `2` on exactly one failure line: the file is one failure, whichever
/// spelling that line names it by (§FS-check.2.4.2).
#[cfg(unix)]
fn assert_one_failure(run: &Run) {
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(run.failure_lines().len(), 1, "{}", run.shown());
}

#[cfg(unix)]
#[test]
fn a_walk_below_a_linked_out_cases_root_and_the_pass_at_it_report_its_ignore_file_once() {
    // The walk from the case meets the file from above at its resolved path, and the pass at the
    // configured folder.
    let (tree, outside) = with_linked_out_cases("cases-met-twice");
    tree.write(
        "grund.toml",
        "grund_config_version = 1\n\n[scan]\ninclude = [\"tests/e2e/1-alpha\"]\n\n\
         [[kinds]]\nkind = \"E2E\"\nfolder = \"tests/e2e\"\n",
    );
    fs::write(outside.join(".ignore"), "{alpha\n").expect("write the ignore file");
    let (plain, full) = (tree.check(), tree.check_with(&["--full"]));
    let _ = fs::remove_dir_all(&outside);
    assert_one_failure(&plain);
    assert_one_failure(&full);
}

#[test]
fn a_git_directory_without_exclude_or_gitignore_is_no_failure() {
    let tree = Tree::new("plain-git", false, "");
    fs::create_dir(tree.0.join(".git")).expect("create the git directory");
    let run = tree.check();
    assert_eq!(run.code, 0, "{}", run.shown());
    assert_eq!(run.failure_lines(), Vec::<&str>::new(), "{}", run.shown());
}

#[test]
fn a_linked_worktree_git_file_is_no_failure() {
    let tree = Tree::new("worktree", false, "");
    tree.write("gitdata/commondir", ".\n");
    let pointer = format!("gitdir: {}\n", tree.0.join("gitdata").display());
    tree.write(".git", &pointer);
    let run = tree.check();
    assert_eq!(run.code, 0, "{}", run.shown());
    assert_eq!(run.failure_lines(), Vec::<&str>::new(), "{}", run.shown());
}
