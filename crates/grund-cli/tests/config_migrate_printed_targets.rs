//! Issue #1458: the `[reference] strict = false` refusal's repair and a
//! `member:` line's command are commands for the user to paste, spelled from the
//! working directory like the undo (§FS-config-migrate.6.5.1). The repair names
//! the refused project's root (§FS-config-migrate.3.1.2), the command the
//! member's root (§FS-config-migrate.8). Ports the reproducer of tool report
//! grund.313: where a shell is at hand, each printed command is pasted where
//! migrate ran and must change the project or member the line names, and no
//! other config.

#[path = "support/temporary_root.rs"]
mod temporary_root;
use temporary_root::TemporaryRoot;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// grund.313's refused project: version 1, `[reference] strict = false`.
const STRICT_FALSE_V1: &str = "grund_config_version = 1

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"AR\"
folder = \"docs/ar\"
index = false

[reference]
strict = false

[scan]
include = [\"docs\"]
";

/// grund.313's member, and the project of the workspace cases.
const MEMBER_V1: &str = "grund_config_version = 1

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

/// The repair line of §FS-config-migrate.3.1.2, naming `dir` (empty, or a space
/// and the project root as printed).
fn repair(dir: &str) -> String {
    format!(
        "  repair: run grund fmt --marker --write{dir}, then set [reference] strict = true \
         under version 1"
    )
}

/// The `member:` line of §FS-config-migrate.8 for the member config `file`,
/// migrated by a command naming `dir` (a space and the directory as printed).
fn member(file: &str, dir: &str) -> String {
    format!("member: {file} is version 1; migrate it with grund config migrate --to 2{dir}")
}

/// A git repository under the system temp directory, every file committed.
struct Tree(TemporaryRoot);

impl Tree {
    fn new(name: &str) -> Self {
        Self(TemporaryRoot::new(&std::env::temp_dir(), name))
    }

    /// A refused project whose config is `config`: `AR-x` cites `AR-y` bare, and
    /// so does `docs/notes.md`, outside the `docs/ar` home.
    fn strict(&self, config: &str) {
        let root = config
            .trim_end_matches("grund.toml")
            .trim_end_matches(".agents/");
        self.write(config, STRICT_FALSE_V1);
        self.write(
            &format!("{root}docs/ar/AR-x.md"),
            "# AR-x: X\n\nSee AR-y for the rest.\n",
        );
        self.write(
            &format!("{root}docs/ar/AR-y.md"),
            "# AR-y: Y\n\nThe rest.\n",
        );
        self.write(&format!("{root}docs/notes.md"), "See AR-y too.\n");
    }

    /// A version-1 workspace root at `root` listing `members`, each a version-1
    /// project with its config at `<member>/<config>` and a `project_name`.
    fn workspace(&self, root: &str, members: &[&str], config: &str) {
        let list = members
            .iter()
            .map(|m| format!("\"{m}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let root_config = MEMBER_V1
            .replace("\"AR\"", "\"FS\"")
            .replace("docs/ar", "docs/fs");
        self.write(
            &format!("{root}grund.toml"),
            &format!("{root_config}\n[workspace]\nmembers = [{list}]\n"),
        );
        self.write(
            &format!("{root}docs/fs/FS-root.md"),
            "# FS-root: Root\n\nThe root project.\n",
        );
        for (i, m) in members.iter().enumerate() {
            // A `project_name` is the alias a directory such as `-x` cannot be.
            let named = MEMBER_V1.replacen('\n', &format!("\nproject_name = \"api{i}\"\n"), 1);
            self.write(&format!("{root}{m}/{config}"), &named);
            self.write(
                &format!("{root}{m}/docs/ar/AR-api.md"),
                "# AR-api: Api\n\nThe member project.\n",
            );
        }
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

    /// Git with no user or system configuration, so no hook or signing applies.
    fn commit(&self) {
        for args in [
            &["init", "-q"][..],
            &["add", "-A"],
            &["commit", "-qm", "version 1"],
        ] {
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
    }

    /// `grund <args>`, run in `cwd` below the root.
    fn grund(&self, cwd: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.path(cwd))
            .output()
            .unwrap()
    }

    /// `grund config migrate --to 2 <args>`, run in `cwd` below the root.
    fn migrate(&self, cwd: &str, args: &[&str]) -> Output {
        let mut all = vec!["config", "migrate", "--to", "2"];
        all.extend_from_slice(args);
        self.grund(cwd, &all)
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
            .output()
            .unwrap()
    }

    /// The repair in `stderr`, pasted in `cwd`: it must mark the bare citation
    /// in each of `marked` and leave `config` as it was.
    #[cfg(unix)]
    fn pastes_repair(&self, cwd: &str, stderr: &str, config: &str, marked: &[&str]) {
        let before = self.read(config);
        let command = printed(stderr, "  repair: run ")
            .split_once(", then set")
            .unwrap()
            .0
            .to_owned();
        let out = self.paste(cwd, &command);
        assert!(out.status.success(), "`{command}`: {}", text(&out.stderr));
        for file in marked {
            assert!(
                self.read(file).contains("\u{a7}AR-y"),
                "`{command}` pasted in `{cwd}` leaves {file} bare: {}{}",
                text(&out.stdout),
                text(&out.stderr)
            );
        }
        assert_eq!(
            self.read(config),
            before,
            "the repair formats, never migrates"
        );
    }

    /// The `member:` command for `file` in `stderr`, pasted in `cwd`: the preview
    /// prints the member's own candidate, and `--write` replaces `config`, the
    /// member's, while every config in `untouched` keeps its bytes.
    #[cfg(unix)]
    fn pastes_member(&self, cwd: &str, stderr: &str, file: &str, config: &str, untouched: &[&str]) {
        let member_root = config
            .trim_end_matches("grund.toml")
            .trim_end_matches(".agents/");
        let command = stderr
            .lines()
            .filter(|line| line.contains(&format!("member: {file} ")))
            .find_map(|line| {
                line.split_once("migrate it with ")
                    .map(|(_, c)| c.to_owned())
            })
            .unwrap_or_else(|| panic!("no member line for {file} in:\n{stderr}"));
        let candidate = text(&self.migrate(member_root, &[]).stdout);
        let preview = self.paste(cwd, &command);
        let reached = match preview.status.code() {
            Some(0) if text(&preview.stdout) == candidate => "the member's candidate".to_owned(),
            Some(0) if text(&preview.stdout).contains("[workspace]") => {
                "the workspace root's candidate".to_owned()
            }
            Some(0) => "another config's candidate".to_owned(),
            code => format!("exit {code:?}: {}", text(&preview.stderr).trim_end()),
        };
        assert_eq!(
            reached, "the member's candidate",
            "`{command}` pasted in `{cwd}` previews the member {file}"
        );
        let before: Vec<String> = untouched.iter().map(|c| self.read(c)).collect();
        let write = self.paste(cwd, &format!("{command} --write"));
        assert_eq!(write.status.code(), Some(0), "{}", text(&write.stderr));
        let after: Vec<String> = untouched.iter().map(|c| self.read(c)).collect();
        assert_eq!(
            after, before,
            "`{command} --write` touches no config but the member's"
        );
        assert!(
            self.read(config).starts_with("grund_config_version = 2"),
            "`{command} --write` pasted in `{cwd}` migrates {config}"
        );
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The rest of the one line of `stderr` that starts with `prefix`.
#[cfg(unix)]
fn printed<'a>(stderr: &'a str, prefix: &str) -> &'a str {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("no `{prefix}` line in:\n{stderr}"))
}

/// The one line of `stderr` that starts with `prefix`, whole.
fn line<'a>(stderr: &'a str, prefix: &str) -> &'a str {
    stderr
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("no `{prefix}` line in:\n{stderr}"))
}

/// A refused write: exit 1, and the config left as it was.
fn refused_write(tree: &Tree, cwd: &str, args: &[&str], config: &str) -> String {
    let before = tree.read(config);
    let out = tree.migrate(cwd, args);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert_eq!(tree.read(config), before);
    text(&out.stderr)
}

/// The refused project at `config`, migrated in `cwd` with `args` and `--write`:
/// where a shell is at hand, its repair pasted in `cwd` marks each of `marked`;
/// and it reads `repair(spelled)`.
fn repair_case(name: &str, config: &str, cwd: &str, args: &[&str], spelled: &str, marked: &[&str]) {
    let tree = Tree::new(&format!("grund-1458-repair-{name}"));
    tree.strict(config);
    tree.commit();
    let mut all = args.to_vec();
    all.push("--write");
    let stderr = refused_write(&tree, cwd, &all, config);
    #[cfg(unix)]
    tree.pastes_repair(cwd, &stderr, config, marked);
    #[cfg(not(unix))]
    let _ = marked;
    assert_eq!(
        line(&stderr, "  repair: "),
        repair(spelled),
        "{name}: the repair names the refused project's root from where migrate ran"
    );
}

/// A workspace at `root` listing `apps/api` with its config at `config`,
/// previewed in `cwd` with `args`: where a shell is at hand, the `member:`
/// command pasted in `cwd` previews and migrates the member alone; and the line
/// names `file`, a report path, and the directory `dir`.
fn member_case(name: &str, root: &str, config: &str, cwd: &str, args: &[&str], dir: &str) {
    let tree = Tree::new(&format!("grund-1458-member-{name}"));
    tree.workspace(root, &["apps/api"], config);
    tree.commit();
    let out = tree.migrate(cwd, args);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stderr = text(&out.stderr);
    let file = format!("apps/api/{config}");
    #[cfg(unix)]
    tree.pastes_member(
        cwd,
        &stderr,
        &file,
        &format!("{root}{file}"),
        &[&format!("{root}grund.toml")],
    );
    assert_eq!(
        line(&stderr, "member: "),
        member(&file, dir),
        "{name}: the file stays a report path; the directory is spelled from where migrate ran"
    );
}

/// §FS-config-migrate.3.1.2, §FS-config-migrate.6.5.1 item 1: grund.313's case
/// A. Migrated from its parent by `PATH` `sub`, the refused project's repair
/// names `sub`, and pasted in the parent it marks the project's bare citations.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_repair_pasted_in_the_parent_marks_the_refused_project() {
    let marked = ["sub/docs/ar/AR-x.md", "sub/docs/notes.md"];
    repair_case("parent", "sub/grund.toml", "", &["sub"], " sub", &marked);
}

/// §FS-config-migrate.3.1.2, §FS-fmt.1: `fmt` narrows to a path it is given, so
/// the repair names the project root, not `PATH`: migrating `sub/docs/ar` names
/// `sub`, and the pasted repair marks `docs/notes.md` outside that path too.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_repair_names_the_project_root_not_the_path() {
    let marked = ["sub/docs/ar/AR-x.md", "sub/docs/notes.md"];
    repair_case(
        "path",
        "sub/grund.toml",
        "",
        &["sub/docs/ar"],
        " sub",
        &marked,
    );
}

/// §FS-config-migrate.6.5.1 item 1: with no `PATH` from `sub/docs`, the
/// project root is `..`. Today's bare repair already formatted the whole
/// project from there, so only the bytes move.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_repair_from_a_project_subdirectory_names_the_root_above() {
    let marked = ["sub/docs/ar/AR-x.md", "sub/docs/notes.md"];
    repair_case("subdir", "sub/grund.toml", "sub/docs", &[], " ..", &marked);
}

/// §FS-config-migrate.6.5.1 item 1: for an `.agents/grund.toml` the repair
/// names the project root, not `.agents/`.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn an_agents_config_repair_names_the_project_root() {
    let marked = ["sub/docs/ar/AR-x.md"];
    repair_case(
        "agents",
        "sub/.agents/grund.toml",
        "",
        &["sub"],
        " sub",
        &marked,
    );
}

/// §FS-config-migrate.6.5.1 item 3: the repair's directory is one POSIX shell
/// word, and one that would start with `-` gets `./`, since grund takes no
/// `PATH` that starts with `-`.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_repair_directory_pastes_as_one_shell_word() {
    let marked = ["my app/docs/ar/AR-x.md"];
    repair_case(
        "space",
        "my app/grund.toml",
        "",
        &["my app"],
        " 'my app'",
        &marked,
    );
    let marked = ["-x/docs/ar/AR-x.md"];
    repair_case("dash", "-x/grund.toml", "", &["./-x"], " ./-x", &marked);
}

/// §FS-config-migrate.8, §FS-config-migrate.6.5.1 item 1: grund.313's case B2.
/// From `ws/docs` with no `PATH`, the pasted command migrates the member, not
/// the root its discovery reaches when the directory is spelled from the root.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_member_command_pasted_below_the_root_migrates_the_member() {
    member_case(
        "subdir",
        "ws/",
        "grund.toml",
        "ws/docs",
        &[],
        " ../apps/api",
    );
}

/// §FS-config-migrate.8, §FS-config-migrate.6.5.1 item 1: grund.313's case B1.
/// From the root's parent with `PATH` `ws`, the pasted command reaches the
/// member instead of failing on a directory that is not there.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_member_command_pasted_above_the_root_migrates_the_member() {
    member_case("parent", "ws/", "grund.toml", "", &["ws"], " ws/apps/api");
}

/// §FS-config-migrate.6.5.1 item 1: for a member's `.agents/grund.toml` the
/// command names the member's root, not `.agents/`.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn an_agents_member_is_named_by_its_project_root() {
    member_case(
        "agents",
        "",
        ".agents/grund.toml",
        "docs",
        &[],
        " ../apps/api",
    );
}

/// §FS-config-migrate.6.5.1 item 3: from the workspace root, a member directory
/// that needs quoting is quoted and one starting with `-` gets `./`, while the
/// files stay report paths.
#[test]
#[ignore = "red until #1458 spells migrate's repair and member: directories from the working directory; implement removes this"]
fn the_member_directories_paste_as_one_shell_word() {
    let tree = Tree::new("grund-1458-word-members");
    tree.workspace("", &["apps/my api", "-x"], "grund.toml");
    tree.commit();
    let out = tree.migrate("", &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stderr = text(&out.stderr);
    #[cfg(unix)]
    {
        let space = "apps/my api/grund.toml";
        let dash = "-x/grund.toml";
        tree.pastes_member("", &stderr, space, space, &["grund.toml", dash]);
        tree.pastes_member("", &stderr, dash, dash, &["grund.toml"]);
    }
    let members: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("member: "))
        .collect();
    assert_eq!(
        members,
        [
            member("-x/grund.toml", " ./-x"),
            member("apps/my api/grund.toml", " 'apps/my api'"),
        ],
        "the files are report paths; the directories are shell words"
    );
}

/// §FS-config-migrate.6.5.1 item 5: at the project root with no `PATH`, the
/// repair names no directory and the member command names the member from the
/// root, the bytes `grund check` there prints after `warning: `
/// (§FS-check.checks.config-version-1). A guard, green before #1458 and after.
#[test]
fn at_the_project_root_with_no_path_the_lines_keep_their_bytes() {
    let marked = ["docs/ar/AR-x.md", "docs/notes.md"];
    repair_case("root", "grund.toml", "", &[], "", &marked);
    member_case("root", "", "grund.toml", "", &[], " apps/api");

    let tree = Tree::new("grund-1458-root-caution");
    tree.workspace("", &["apps/api"], "grund.toml");
    tree.commit();
    let migrate = text(&tree.migrate("", &[]).stderr);
    let check = text(&tree.grund("", &["check"]).stderr);
    assert_eq!(
        line(&check, "warning: apps/api/").strip_prefix("warning: "),
        line(&migrate, "member: ").strip_prefix("member: "),
        "the caution and the member line agree at the root"
    );
}
