//! Black-box: a link whose resolution loops is a loop whatever its name (§FS-scan.scope.links.owed.cycle),
//! so a self-link or a cycle of links met on the walk is reported at the link and exits `2`
//! (§FS-scan.scope.links.unresolvable), and naming one as the path reports it rather than calling it
//! missing (§FS-scan.scope.links.unresolvable.named). The controls hold what must not move: the
//! extensionless dangling link stays the declared blind spot (§FS-scan.scope.links.blind-spot), and
//! a looping link the ignore files or a hidden name keep out of the walk stays silent.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

/// A version-2 project whose one kind keeps its home at `docs/`, every ID there cited.
struct Tree(PathBuf);

impl Tree {
    fn new(test: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1331-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the kind home");
        fs::write(
            root.join("grund.toml"),
            "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
             slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\
             index = false\n",
        )
        .expect("write the config");
        fs::write(
            root.join("docs/a.md"),
            "# FS-a: A\n\nText, see \u{a7}FS-a.\n",
        )
        .expect("write");
        Self(root)
    }

    /// Creates `docs/<link>` pointing at `target` as written.
    fn link(self, link: &str, target: &str) -> Self {
        symlink(target, self.0.join("docs").join(link)).expect("create the link");
        self
    }

    fn write(self, relative: &str, content: &str) -> Self {
        fs::write(self.0.join(relative), content).expect("write the file");
        self
    }

    /// Adds a v2 `[schema.sources] exclude` listing `patterns` to the config.
    fn exclude(self, patterns: &[&str]) -> Self {
        let config = self.0.join("grund.toml");
        let mut text = fs::read_to_string(&config).expect("read the config");
        let listed: Vec<String> = patterns.iter().map(|p| format!("\"{p}\"")).collect();
        text.push_str(&format!(
            "\n[schema.sources]\nexclude = [{}]\n",
            listed.join(", ")
        ));
        fs::write(&config, text).expect("write the config");
        self
    }

    /// Makes the tree a git repository, so its `.gitignore` is read.
    fn git(self) -> Self {
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&self.0)
            .status()
            .expect("run git init");
        assert!(status.success(), "git init failed");
        self
    }

    fn grund(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        Run {
            shown: format!("grund {}", args.join(" ")),
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn errors(&self) -> Vec<&str> {
        let lines = self.stdout.lines().chain(self.stderr.lines());
        lines.filter(|line| line.starts_with("error: ")).collect()
    }

    /// Exit `2`, and for each link one error line at that link's own path that calls it a loop.
    fn assert_reports(&self, links: &[&str]) {
        assert_eq!(self.code, 2, "{}", self.report());
        for link in links {
            let at = format!("error: {link}: ");
            assert!(
                self.errors()
                    .iter()
                    .any(|line| line.starts_with(&at) && line.contains("loop")),
                "no loop reported at {link}\n{}",
                self.report()
            );
        }
        assert!(
            !self.stderr.contains("does not exist"),
            "a link that exists was called missing\n{}",
            self.report()
        );
    }

    fn assert_silent(&self) {
        assert_eq!(self.code, 0, "{}", self.report());
        assert!(self.errors().is_empty(), "{}", self.report());
    }

    fn report(&self) -> String {
        let Self {
            shown,
            code,
            stdout,
            stderr,
        } = self;
        format!("{shown}: exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    }
}

/// The ticket's `self`, `self-full` and `grund check docs`: the walk meets the link on the way down.
#[test]
fn an_extensionless_self_link_is_reported_at_the_link() {
    let tree = Tree::new("self").link("self", "self");
    for args in [&["check"][..], &["check", "--full"], &["check", "docs"]] {
        tree.grund(args).assert_reports(&["docs/self"]);
    }
}

/// The ticket's `pair`: each link of a two-link cycle is a loop.
#[test]
fn each_link_of_an_extensionless_cycle_is_reported() {
    let tree = Tree::new("pair").link("a2", "b").link("b", "a2");
    tree.grund(&["check"])
        .assert_reports(&["docs/a2", "docs/b"]);
}

/// `extensions` is never asked of a loop, so a name it does not list is reported too.
#[test]
fn a_self_link_with_an_unlisted_extension_is_reported() {
    let tree = Tree::new("png").link("logo.png", "logo.png");
    tree.grund(&["check"]).assert_reports(&["docs/logo.png"]);
}

/// A looping link named as the path exists, so the run reports it rather than calling it missing.
#[test]
fn a_looping_link_named_as_the_path_is_reported_not_missing() {
    let tree = Tree::new("named")
        .link("self", "self")
        .link("self.md", "self.md");
    tree.grund(&["check", "docs/self"])
        .assert_reports(&["docs/self"]);
    tree.grund(&["check", "docs/self.md"])
        .assert_reports(&["docs/self.md"]);
}

/// Control: the ticket's `mdself`, reported today and after.
#[test]
fn a_self_link_with_a_source_extension_is_reported() {
    let tree = Tree::new("mdself").link("self.md", "self.md");
    tree.grund(&["check"]).assert_reports(&["docs/self.md"]);
}

/// Control: the ticket's `walk`, a link back to its own directory.
#[test]
fn a_link_to_its_own_directory_is_reported() {
    let tree = Tree::new("walk").link("up", ".");
    tree.grund(&["check"]).assert_reports(&["docs/up"]);
}

/// Control: a dangling extensionless link is the declared blind spot and stays silent.
#[test]
fn a_dangling_extensionless_link_stays_silent() {
    let tree = Tree::new("dangling").link("shared", "nowhere");
    tree.grund(&["check"]).assert_silent();
    tree.grund(&["check", "--full"]).assert_silent();
}

/// Control: an ignore file covering a looping link keeps it out of the walk, as a file pattern or
/// as a directory one, since a loop is judged as a directory.
#[test]
fn an_ignored_looping_link_stays_silent() {
    for pattern in ["docs/self\n", "docs/self/\n"] {
        let tree = Tree::new("ignored")
            .link("self", "self")
            .write(".ignore", pattern);
        tree.grund(&["check"]).assert_silent();
    }
}

/// Control: a hidden name keeps a looping link out of the walk.
#[test]
fn a_hidden_looping_link_stays_silent() {
    let tree = Tree::new("hidden").link(".self", ".self");
    tree.grund(&["check"]).assert_silent();
}

/// A loop is judged by the ignore files as a directory and only as one: a `.gitignore` that drops
/// every entry of `docs/` but keeps its directories keeps a looping link, which is then owed a
/// report whatever its name (§FS-scan.scope.links.owed.cycle).
#[test]
fn a_looping_link_kept_as_a_directory_is_reported() {
    let tree = Tree::new("negated")
        .git()
        .link("self", "self")
        .link("logo.png", "logo.png")
        .write(".gitignore", "docs/*\n!docs/*/\n!docs/*.md\n");
    tree.grund(&["check"])
        .assert_reports(&["docs/self", "docs/logo.png"]);
}

/// A link back to its own directory covered only by a directory pattern is dropped as a
/// directory, so it stays silent (§FS-scan.scope.links.owed.cycle).
#[test]
fn a_link_to_its_own_directory_ignored_as_a_directory_stays_silent() {
    let tree = Tree::new("walk-ignored")
        .git()
        .link("up", ".")
        .write(".gitignore", "docs/up/\n");
    tree.grund(&["check"]).assert_silent();
}

/// A v2 `exclude` pattern keeps a looping link out as it keeps out a directory, whichever way the
/// loop is found: a self-link, a link whose name `extensions` does not list, and a link back to its
/// own directory (§FS-scan.scope.links.owed.cycle).
#[test]
fn a_looping_link_a_v2_exclude_covers_stays_silent() {
    let cases: [(&str, &str, &[&str]); 4] = [
        ("logo.png", "logo.png", &["*.png"]),
        ("self", "self", &["docs/self"]),
        ("self", "self", &["**/self"]),
        ("up", ".", &["**/up"]),
    ];
    for (link, target, patterns) in cases {
        let tree = Tree::new("excluded").link(link, target).exclude(patterns);
        tree.grund(&["check"]).assert_silent();
    }
}

/// Control: an exclude that covers some other path leaves a looping link reported.
#[test]
fn a_looping_link_no_exclude_covers_is_reported() {
    let tree = Tree::new("excluded-other")
        .link("self", "self")
        .link("up", ".")
        .exclude(&["*.png", "docs/other"]);
    tree.grund(&["check"])
        .assert_reports(&["docs/self", "docs/up"]);
}
