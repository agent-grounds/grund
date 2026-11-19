//! A row closed to every citation (§FS-config.3.9.4.1) renders its Citation directions bullet as
//! one unnarrowed sentence with no grounding hint, `- Each file in **ce/** must not cite anything.`,
//! in the root block a version-1 config writes and the scoped block a version-2 config writes
//! (§FS-init.2.3.5.2.2), and the homeless kind closed the same way is not narrowed either
//! (§FS-init.2.3.5.2.1). The text moves under one block-version bump (§FS-init.2.3.7.1), and a
//! version-1 closed row still migrates with both gates equal (§FS-config-migrate.5.3.1).
//! Issue #1456 is the report: the block read `that cites anything must not cite anything` and
//! named a grounding setting under which no file there can pass.
//!
//! Each case first proves its row closed: a citation in its source file is refused by `grund
//! check` as a citation direction. The trees are built in the temp directory, so run them with a
//! `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The version-1 head: `FS` declared in `specification/`.
const V1_HEAD: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
                       [[kinds]]\nkind = \"FS\"\nfolder = \"specification\"\nindex = false\n";

/// The version-2 head, the same kind.
const V2_HEAD: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.kinds.FS]\n\
                       folders = [\"specification\"]\nindex = false\n";

/// The non-citable `ce/` home in version 1, closed by its `must-not` default; `{grounding}` is a
/// line under the row.
const V1_CE: &str = "\n[[kinds]]\nkind = \"ce\"\nfolder = \"ce\"\ncitable = false\n{grounding}\n\
                     [citations.ce]\ndefault = \"must-not\"\n";

/// The same home in version 2; `{grounding}` is the row's ladder.
const V2_CE: &str = "\n[schema.kinds.ce]\nfolders = [\"ce\"]\ncitable = false\n\n\
                     [rules.citations.ce]\ndefault = \"must-not\"\n{grounding}";

/// The bullet the closed `ce/` home renders, whatever its grounding.
const CLOSED_HOME: &str = "- Each file in **ce/** must not cite anything.";

/// The bullet the closed homeless kind renders.
const CLOSED_CODE: &str =
    "- Each source file outside the Project map (**code**) must not cite anything.";

/// The complete report's line when both gates pass (§FS-config-migrate.5.4).
const COMPARED: &str = "\ncompared: the records are equal, and grund check --suggestions on this \
                        tree gives the same findings and the same exit before and after\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// `config`, `FS-login` in `specification/`, an uncited `source` file, a git repository, and
    /// the blocks `grund init` writes.
    fn new(name: &str, config: &str, source: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1456-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let files = [
            ("grund.toml", config),
            (
                "specification/FS-login.md",
                "# FS-login: Login\n\nUsers log in.\n",
            ),
            (source, "class A {}\n"),
        ];
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
            fs::write(path, text).expect("write a file");
        }
        let tree = Self(root);
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&tree.0)
            .status()
            .expect("run git");
        assert!(git.success(), "git init");
        tree.init();
        tree
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    fn init(&self) {
        let init = self.run(&["init"]);
        assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
    }

    /// The bullet `file` renders for the line opening `- <subject>`, or the whole file.
    fn bullet<'a>(&self, text: &'a str, subject: &str) -> &'a str {
        let opening = format!("- {subject}");
        text.lines()
            .find(|line| line.starts_with(&opening))
            .unwrap_or(text)
    }

    /// The control: a citation of `FS-login` in `source` is refused as a citation direction, so
    /// the row is closed. The file is restored after.
    fn assert_closed(&self, source: &str) {
        let path = self.0.join(source);
        let saved = fs::read_to_string(&path).expect("read the source");
        fs::write(&path, format!("// \u{a7}FS-login\n{saved}")).expect("cite");
        let check = self.run(&["check"]);
        fs::write(&path, saved).expect("restore");
        let stdout = String::from_utf8_lossy(&check.stdout);
        assert!(
            stdout
                .lines()
                .any(|line| line.starts_with(&format!("{source}:1: error:"))
                    && line.contains("must not cite FS (citation direction)")),
            "{}",
            streams(&check)
        );
    }

    /// Rewrites the version on every managed block heading in `files` to `version`.
    fn set_block_version(&self, files: &[&str], version: u32) {
        for file in files {
            let text = self.read(file);
            let start = text.find("(block v").expect("a block heading") + "(block v".len();
            let end = start + text[start..].find(')').expect("a closing parenthesis");
            let rewritten = format!("{}{version}{}", &text[..start], &text[end..]);
            fs::write(self.0.join(file), rewritten).expect("write the block");
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The two streams of `output`, for an assertion message.
fn streams(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The version-1 config with the closed `ce/` home, `grounding` under its row.
fn v1_closed(grounding: &str) -> String {
    format!("{V1_HEAD}{}", V1_CE.replace("{grounding}", grounding))
}

/// The version-2 config with the closed `ce/` home, `grounding` as its ladder.
fn v2_closed(grounding: &str) -> String {
    format!("{V2_HEAD}{}", V2_CE.replace("{grounding}", grounding))
}

/// [`v1_closed`] with nothing for `config migrate` to refuse: named sections on and an explicit
/// scan, as §FS-config-migrate.3.1.1 and §FS-config-migrate.3.1.3 ask of a version-1 config.
fn v1_migratable() -> String {
    let config = v1_closed("").replace(
        "format = \"{kind}-{slug}\"\n",
        "format = \"{kind}-{slug}\"\nnamed_sections = true\n",
    );
    format!("{config}\n[scan]\ninclude = [\"specification\", \"ce\"]\n")
}

/// §FS-init.2.3.5.2.2: issue #1456's case A1. A version-1 closed home that grounds nothing renders
/// one sentence in the root block, no narrowing and no hint. It used to read `Each file in **ce/**
/// that cites anything must not cite anything. A file there that cites nothing is not checked;
/// `require_grounding = true` on its `[[kinds]]` row makes it a finding.`
#[test]
fn a_v1_closed_home_renders_one_sentence_in_the_root_block() {
    let tree = Tree::new("a1", &v1_closed(""), "ce/A.java");
    tree.assert_closed("ce/A.java");
    let agents = tree.read("AGENTS.md");
    assert_eq!(tree.bullet(&agents, "Each file in **ce/**"), CLOSED_HOME);
    assert!(!agents.contains("makes it a finding"), "{agents}");
}

/// §FS-init.2.3.5.2.2: case A2. A version-2 closed home that grounds nothing renders the same
/// sentence in its scoped block. It used to name `[rules.citations.ce.grounding] must = "file"`.
#[test]
fn a_v2_closed_home_renders_one_sentence_in_its_scoped_block() {
    let tree = Tree::new("a2", &v2_closed(""), "ce/A.java");
    tree.assert_closed("ce/A.java");
    let scoped = tree.read("ce/AGENTS.md");
    assert_eq!(tree.bullet(&scoped, "Each file in **ce/**"), CLOSED_HOME);
    assert!(!scoped.contains("makes it a finding"), "{scoped}");
}

/// §FS-init.2.3.5.2.1: cases B1 and B2. The homeless kind closed by its own `must-not` default,
/// with no row declared, is not narrowed in either version. It used to read `Each source file
/// outside the Project map (**code**) that cites anything must not cite anything.`
#[test]
fn the_closed_homeless_kind_renders_one_sentence() {
    for (name, config) in [
        (
            "b1",
            format!("{V1_HEAD}\n[citations.code]\ndefault = \"must-not\"\n"),
        ),
        (
            "b2",
            format!("{V2_HEAD}\n[rules.citations.code]\ndefault = \"must-not\"\n"),
        ),
    ] {
        let tree = Tree::new(name, &config, "src/A.java");
        tree.assert_closed("src/A.java");
        let agents = tree.read("AGENTS.md");
        let subject = "Each source file outside the Project map";
        assert_eq!(tree.bullet(&agents, subject), CLOSED_CODE, "{name}");
    }
}

/// §FS-init.2.3.5.2.2: cases C1 and C2, the bullet only. A closed home whose grounding is on
/// already renders the one sentence, and keeps it. A guard: it holds today. That such a config
/// loads in silence is agent-grounds/grund#1460's, and no test here pins it.
#[test]
fn a_grounded_closed_home_keeps_its_bullet() {
    for (name, config, scoped) in [
        ("c1", v1_closed("require_grounding = true\n"), "AGENTS.md"),
        (
            "c2",
            v2_closed("\n[rules.citations.ce.grounding]\nmust = \"file\"\n"),
            "ce/AGENTS.md",
        ),
    ] {
        let tree = Tree::new(name, &config, "ce/A.java");
        tree.assert_closed("ce/A.java");
        let agents = tree.read(scoped);
        assert_eq!(
            tree.bullet(&agents, "Each file in **ce/**"),
            CLOSED_HOME,
            "{name}"
        );
    }
}

/// §FS-init.2.3.7.1, §FS-init.2.3.7.3.2: the block the previous release wrote, v47 at the root of a
/// version-1 config and v49 in each block of a version-2 config with a scoped file, is outdated,
/// and `grund check` names both versions; `grund init` repairs it. Before the bump v47 and v49 were
/// current, and `grund check` reported nothing.
#[test]
fn the_previous_release_block_is_outdated() {
    for (name, config, files, older, selected) in [
        ("outdated-v1", v1_closed(""), &["AGENTS.md"][..], 47, 51),
        (
            "outdated-v2",
            v2_closed(""),
            &["AGENTS.md", "ce/AGENTS.md"][..],
            49,
            53,
        ),
    ] {
        let tree = Tree::new(name, &config, "ce/A.java");
        tree.set_block_version(files, older);
        let check = tree.run(&["check"]);
        let stdout = String::from_utf8_lossy(&check.stdout);
        let outdated = format!(
            "outdated grund init block v{older} — run `grund init` to update to v{selected}"
        );
        assert!(stdout.contains(&outdated), "{name}: {}", streams(&check));
        assert_eq!(check.status.code(), Some(1), "{name}: {}", streams(&check));
        tree.init();
        for file in files {
            let text = tree.read(file);
            assert!(
                text.contains(&format!("(block v{selected})")),
                "{name} {file}: {text}"
            );
        }
        let check = tree.run(&["check"]);
        assert_eq!(check.status.code(), Some(0), "{name}: {}", streams(&check));
    }
}

/// §FS-config-migrate.5.3.1: a version-1 closed home that grounds nothing migrates with both gates
/// equal. A guard: it holds today.
#[test]
fn config_migrate_passes_a_closed_row() {
    let tree = Tree::new("migrate", &v1_migratable(), "ce/A.java");
    let output = tree.run(&["config", "migrate", "--to", "2"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(COMPARED) && !stderr.contains("refused:") && !stderr.contains("differs:"),
        "{}",
        streams(&output)
    );
    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
}

/// §FS-config-migrate.5.3.1, §FS-init.2.3.5.2.2: the migrated tree renders the closed home as the
/// version-1 tree does, one sentence and no hint. It used to name the version-2 ladder.
#[test]
fn a_migrated_closed_home_renders_one_sentence() {
    let tree = Tree::new("migrated", &v1_migratable(), "ce/A.java");
    let output = tree.run(&["config", "migrate", "--to", "2", "--write", "--full"]);
    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
    assert!(
        tree.read("grund.toml")
            .starts_with("grund_config_version = 2\n")
    );
    tree.init();
    let mut blocks = tree.read("AGENTS.md");
    if tree.0.join("ce/AGENTS.md").exists() {
        blocks.push_str(&tree.read("ce/AGENTS.md"));
    }
    assert_eq!(tree.bullet(&blocks, "Each file in **ce/**"), CLOSED_HOME);
    assert!(!blocks.contains("makes it a finding"), "{blocks}");
}
