//! Issue #1438: after a write, `grund config migrate --to 2`'s undo and
//! re-preview are commands spelled from the working directory, the re-preview
//! carrying `PATH` as given, while `wrote` stays a report path on the report base
//! (§FS-config-migrate.6.5.1, §FS-config-migrate.2.2). Ports the reproducer of
//! tool report grund.305: where a shell is at hand, each printed command is
//! pasted where it was printed and must act on the config the run replaced.

#[path = "support/temporary_root.rs"]
mod temporary_root;
use temporary_root::TemporaryRoot;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The workspace root of grund.305: 16 lines, listing `apps/api`.
const ROOT_V1: &str = "grund_config_version = 1

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"FS\"
folder = \"docs/fs\"
index = false

[scan]
include = [\"docs\"]

[workspace]
members = [\"apps/api\"]
";

/// The member of grund.305, and the lone project of the other cases: 13 lines.
const PROJECT_V1: &str = "grund_config_version = 1

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"AR\"
folder = \"docs/ar\"
index = false

[scan]
include = [\"docs\"]
";

const PROJECT_MAP: &str = "note: the next grund init adds a `code` row to the AGENTS.md \
Project map, because the complement's places are now listed; grund check reports the block \
stale until then";
const UNDO: &str = "to get version 1 back: it is in version control, e.g. git restore ";
const REPREVIEW: &str =
    "to see this candidate again: restore version 1 and run grund config migrate --to 2";
const REMINDER: &str = "note: existing AGENTS.md managed instructions may still describe \
version 1; run grund init in the migrated project to refresh them";

/// A git repository under the system temp directory, every file committed.
struct Tree(TemporaryRoot);

impl Tree {
    /// grund.305's tree: a version-1 root listing the version-1 member `apps/api`.
    fn workspace(name: &str) -> Self {
        let tree = Self(TemporaryRoot::new(&std::env::temp_dir(), name));
        tree.write("grund.toml", ROOT_V1);
        tree.write(
            "docs/fs/FS-root.md",
            "# FS-root: Root\n\nThe root project.\n",
        );
        tree.project("apps/api");
        tree.commit();
        tree
    }

    /// One version-1 project at `dir` below the repository root, and no other.
    fn single(name: &str, dir: &str) -> Self {
        let tree = Self(TemporaryRoot::new(&std::env::temp_dir(), name));
        tree.project(dir);
        tree.commit();
        tree
    }

    fn project(&self, dir: &str) {
        let dir = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        self.write(&format!("{dir}grund.toml"), PROJECT_V1);
        self.write(
            &format!("{dir}docs/ar/AR-api.md"),
            "# AR-api: Api\n\nThe project.\n",
        );
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap()
    }

    /// The `grund_config_version` a config holds now.
    #[cfg(unix)]
    fn version(&self, config: &str) -> String {
        self.read(config)
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    /// Git with no user or system configuration, so no hook or signing applies.
    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .args([
                "-c",
                "user.name=grund",
                "-c",
                "user.email=grund@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .current_dir(&*self.0)
            .env("GIT_CONFIG_GLOBAL", self.path(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    }

    fn commit(&self) {
        self.git(&["init", "-q"]);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "version 1"]);
    }

    /// `grund config migrate --to 2 <args>`, run in `cwd` below the root.
    fn migrate(&self, cwd: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["config", "migrate", "--to", "2"])
            .args(args)
            .current_dir(self.path(cwd))
            .output()
            .unwrap()
    }

    /// A command line exactly as grund printed it, pasted into a POSIX shell in
    /// `cwd`, with this build's `grund` first on `PATH`.
    #[cfg(unix)]
    fn paste(&self, cwd: &str, line: &str) -> Output {
        let bin = PathBuf::from(env!("CARGO_BIN_EXE_grund"));
        let path =
            std::env::join_paths(std::iter::once(bin.parent().unwrap().to_path_buf()).chain(
                std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
            ))
            .unwrap();
        Command::new("sh")
            .args(["-c", line])
            .current_dir(self.path(cwd))
            .env("PATH", path)
            .env("GIT_CONFIG_GLOBAL", self.path(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap()
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The compact report of a passing write (§FS-config-migrate.6.5): `wrote` from
/// the report base, then the undo naming `undo` and the re-preview ending in
/// `repreview` (empty, or a space and `PATH`).
fn after_write(wrote: &str, lines: usize, undo: &str, repreview: &str) -> String {
    format!(
        "wrote {wrote}: version 1 ({lines} lines) replaced by version 2\n{PROJECT_MAP}\n\
         {UNDO}{undo}\n{REPREVIEW}{repreview}\n{REMINDER}\n"
    )
}

fn passed(out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}

/// The printed command after `prefix`: the undo's `git restore …`, or the
/// re-preview's `grund config migrate --to 2 …`.
#[cfg(unix)]
fn command(stderr: &str, prefix: &str) -> String {
    let line = stderr
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("no `{prefix}` line in:\n{stderr}"));
    match prefix {
        UNDO => format!("git restore {line}"),
        _ => format!("grund config migrate --to 2{line}"),
    }
}

/// §FS-config-migrate.6.5.1, §FS-config-migrate.8: the ticket's case. The root
/// is migrated and left uncommitted, then its member is migrated from the root
/// by `PATH`. The undo names the member from the root, and pasted there it
/// restores the member alone; the re-preview carries `apps/api`, and pasted at
/// the root it previews the member. `wrote` stays on the member's own root.
#[test]
fn member_migrated_from_the_workspace_root_names_it_from_the_root() {
    let tree = Tree::workspace("grund-1438-member");
    passed(&tree.migrate("", &["--write"]));
    let write = tree.migrate("", &["apps/api", "--write"]);
    passed(&write);
    let stderr = text(&write.stderr);
    assert_eq!(
        stderr,
        after_write("grund.toml", 13, "apps/api/grund.toml", " apps/api"),
        "the undo and re-preview are spelled from the root; wrote stays a report path"
    );
    #[cfg(unix)]
    {
        let candidate = tree.read("apps/api/grund.toml");
        let undo = tree.paste("", &command(&stderr, UNDO));
        assert!(undo.status.success(), "{}", text(&undo.stderr));
        assert_eq!(
            (
                tree.version("grund.toml"),
                tree.version("apps/api/grund.toml")
            ),
            (
                "grund_config_version = 2".to_owned(),
                "grund_config_version = 1".to_owned()
            ),
            "the printed undo restores the member and leaves the root's migration"
        );
        let again = tree.paste("", &command(&stderr, REPREVIEW));
        passed(&again);
        assert_eq!(
            text(&again.stdout),
            candidate,
            "the re-preview shows the member's candidate"
        );
    }
}

/// §FS-config-migrate.6.5.1: not a workspace matter. A lone project at `sub`,
/// migrated from its parent, is named from the parent and previewed again by
/// the same `PATH`.
#[test]
fn project_below_the_working_directory_is_named_from_it() {
    let tree = Tree::single("grund-1438-sub", "sub");
    let write = tree.migrate("", &["sub", "--write"]);
    passed(&write);
    let stderr = text(&write.stderr);
    assert_eq!(
        stderr,
        after_write("grund.toml", 13, "sub/grund.toml", " sub")
    );
    #[cfg(unix)]
    {
        let candidate = tree.read("sub/grund.toml");
        let undo = tree.paste("", &command(&stderr, UNDO));
        assert!(undo.status.success(), "{}", text(&undo.stderr));
        assert_eq!(tree.version("sub/grund.toml"), "grund_config_version = 1");
        let again = tree.paste("", &command(&stderr, REPREVIEW));
        passed(&again);
        assert_eq!(text(&again.stdout), candidate);
    }
}

/// §FS-config-migrate.6.5.1 item 1: with no `PATH`, run from a directory below
/// the project root, the undo climbs to the config with `..` and the
/// re-preview stays bare, so the same discovery walks up to the same file.
#[test]
fn no_path_from_a_subdirectory_restores_the_config_above() {
    let tree = Tree::single("grund-1438-subdir", "");
    let write = tree.migrate("docs", &["--write"]);
    passed(&write);
    let stderr = text(&write.stderr);
    assert_eq!(stderr, after_write("grund.toml", 13, "../grund.toml", ""));
    #[cfg(unix)]
    {
        let undo = tree.paste("docs", &command(&stderr, UNDO));
        assert!(undo.status.success(), "{}", text(&undo.stderr));
        assert_eq!(tree.version("grund.toml"), "grund_config_version = 1");
    }
}

/// §FS-config-migrate.6.5.1 item 4: `--path-base` moves `wrote`, a report path,
/// and neither command. The ticket's member reads the same either way; a
/// project's `docs` from its root tells the two apart, `wrote ../grund.toml`
/// from the invocation base beside `git restore grund.toml` from the root.
#[test]
fn path_base_moves_wrote_and_neither_command() {
    for base in ["project", "invocation"] {
        let tree = Tree::workspace(&format!("grund-1438-base-{base}"));
        passed(&tree.migrate("", &["--write"]));
        let write = tree.migrate("", &["apps/api", "--write", "--path-base", base]);
        passed(&write);
        assert_eq!(
            text(&write.stderr),
            after_write("grund.toml", 13, "apps/api/grund.toml", " apps/api"),
            "--path-base {base}"
        );

        let tree = Tree::single(&format!("grund-1438-docs-{base}"), "");
        let write = tree.migrate("", &["docs", "--write", "--path-base", base]);
        passed(&write);
        let wrote = if base == "project" {
            "grund.toml"
        } else {
            "../grund.toml"
        };
        assert_eq!(
            text(&write.stderr),
            after_write(wrote, 13, "grund.toml", " docs"),
            "--path-base {base}"
        );
    }
}

/// §FS-config-migrate.6.5.1 item 5, §FS-config-migrate.1.3: `--write --full`
/// closes with the same outcome and the same two commands after the complete
/// report, and restores the candidate on stdout.
#[test]
fn full_write_prints_the_same_commands() {
    let tree = Tree::workspace("grund-1438-full");
    passed(&tree.migrate("", &["--write"]));
    let write = tree.migrate("", &["apps/api", "--write", "--full"]);
    passed(&write);
    assert_eq!(text(&write.stdout), tree.read("apps/api/grund.toml"));
    let stderr = text(&write.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    let undo = format!("{UNDO}apps/api/grund.toml");
    let repreview = format!("{REPREVIEW} apps/api");
    assert_eq!(
        lines[lines.len().saturating_sub(4)..].to_vec(),
        [
            "wrote grund.toml: version 1 (13 lines) replaced by version 2",
            undo.as_str(),
            repreview.as_str(),
            REMINDER,
        ],
        "{stderr}"
    );
}

/// §FS-config-migrate.6.5.1 item 3: each path is one POSIX shell word. A path
/// with a space or a `'` is single-quoted, `'` written `'\''`; an undo path
/// that would start with `-` gets `./`; a `PATH` of safe characters stays bare.
#[test]
fn paths_paste_as_one_shell_word() {
    for (name, dir, path, undo, repreview) in [
        (
            "space",
            "my app",
            "my app",
            "'my app/grund.toml'",
            " 'my app'",
        ),
        (
            "quote",
            "it's",
            "it's",
            r"'it'\''s/grund.toml'",
            r" 'it'\''s'",
        ),
        ("dash", "-x", "./-x", "./-x/grund.toml", " ./-x"),
    ] {
        let tree = Tree::single(&format!("grund-1438-quote-{name}"), dir);
        let write = tree.migrate("", &[path, "--write"]);
        passed(&write);
        let stderr = text(&write.stderr);
        assert_eq!(
            stderr,
            after_write("grund.toml", 13, undo, repreview),
            "{name}"
        );
        #[cfg(unix)]
        {
            let config = format!("{dir}/grund.toml");
            let candidate = tree.read(&config);
            let restored = tree.paste("", &command(&stderr, UNDO));
            assert!(
                restored.status.success(),
                "{name}: {}",
                text(&restored.stderr)
            );
            assert_eq!(tree.version(&config), "grund_config_version = 1", "{name}");
            let again = tree.paste("", &command(&stderr, REPREVIEW));
            passed(&again);
            assert_eq!(text(&again.stdout), candidate, "{name}");
        }
    }
}
