//! Black-box regression for #824: maintenance already works, but the installed
//! help and freshly generated guidance must teach it (§FS-cli.2.2,
//! §FS-cli.2.3.3, §FS-init.2.3.4.9). Ported from the ticket-local reproducer.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-824-teaching-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("grund.toml"),
            "grund_config_version = 2\nproject_name = \"rename-reproduction\"\n\n\
             [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n\n\
             [presentation.fmt]\nanchors = \"github\"\nlinks = \"index\"\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/functional-spec/FS-login.md"),
            "# FS-login: A user signs in\n\nThe login contract.\n\n\
             ## code: Code\n\n### .1: Lifetime\n\nA session expires after one hour.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/notes.md"),
            "See \u{a7}FS-login and \u{a7}FS-login.code.1.\n",
        )
        .unwrap();
        fs::write(
            root.join("src/a.py"),
            "# The session follows \u{a7}FS-login.code.1.\n",
        )
        .unwrap();
        Self(root)
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn grund(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run the built CLI");
    assert!(output.status.success(), "grund {args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn mentions_move(text: &str) -> bool {
    text.split(|c: char| !c.is_ascii_alphabetic()).any(|word| {
        word.starts_with("renam") || ["move", "moves", "moved", "moving"].contains(&word)
    })
}

fn mentions_locks(text: &str) -> bool {
    text.contains("section lock") || text.contains("section-lock")
}

fn require(surface: &str, text: &str, concepts: &[(&str, bool)]) {
    let missing: Vec<_> = concepts
        .iter()
        .filter_map(|(name, present)| (!present).then_some(*name))
        .collect();
    assert!(
        missing.is_empty(),
        "{surface} omits: {}\n{text}",
        missing.join(", ")
    );
}

/// §FS-cli.2.2: both maintenance effects are discoverable within one screen.
#[test]
#[ignore = "red until #824 top-level fmt help names links and section locks; implement removes this"]
fn top_level_fmt_summary_names_link_and_section_lock_maintenance() {
    let page = grund(Path::new(env!("CARGO_MANIFEST_DIR")), &["--help"]);
    assert!(
        page.lines().count() <= 24,
        "top-level help exceeds 24 lines"
    );
    assert!(page.lines().all(|l| l.chars().count() <= 100));
    let row = page
        .lines()
        .find(|line| line.split_whitespace().next() == Some("fmt"))
        .expect("top-level fmt row")
        .to_lowercase();
    require(
        "top-level fmt summary",
        &row,
        &[
            (
                "links",
                row.contains("link") || row.contains("cross-reference"),
            ),
            ("section locks", mentions_locks(&row)),
        ],
    );
}

/// §FS-cli.2.3.3: the full page teaches the remedy and a practical invocation.
#[test]
#[ignore = "red until #824 fmt help teaches root-scoped rename maintenance; implement removes this"]
fn fmt_help_teaches_root_scoped_post_rename_maintenance() {
    let page = grund(Path::new(env!("CARGO_MANIFEST_DIR")), &["fmt", "--help"]);
    let text = page.to_lowercase();
    let example = text.lines().any(|line| {
        line.contains("grund fmt --write") && line.contains('#') && mentions_move(line)
    });
    let root_scoped_locks = text
        .split("\n\n")
        .any(|paragraph| mentions_locks(paragraph) && paragraph.contains("root"));
    require(
        "fmt help",
        &page,
        &[
            ("declaration rename/move", mentions_move(&text)),
            (
                "generated destinations and anchors",
                text.contains("destination") && text.contains("anchor"),
            ),
            ("section locks", mentions_locks(&text)),
            ("root scope for locks", root_scoped_locks),
            ("post-rename --write example", example),
        ],
    );
}

/// §FS-init.2.3.4.9, §FS-init.2.3.4.3: inspect actual init output, keeping
/// the remedy near rename guidance rather than counting an unrelated --write.
#[test]
#[ignore = "red until #824 generated refs guidance teaches the rename workflow; implement removes this"]
fn generated_guidance_connects_refs_to_rename_maintenance_and_verification() {
    let fixture = Fixture::new();
    grund(&fixture.0, &["init", "--no-vcs", "--agents-md"]);
    let page = fs::read_to_string(fixture.0.join("AGENTS.md")).unwrap();
    let lines: Vec<_> = page.lines().collect();
    let mut context = String::new();
    for (index, line) in lines.iter().enumerate() {
        if mentions_move(&line.to_lowercase()) {
            context
                .push_str(&lines[index.saturating_sub(3)..(index + 4).min(lines.len())].join("\n"));
            context.push('\n');
        }
    }
    let text = context.to_lowercase();
    require(
        "generated rename guidance",
        &context,
        &[
            ("refs before rename", text.contains("grund refs")),
            (
                "author updates citation IDs",
                text.contains("citation")
                    && text.contains("id")
                    && (text.contains("edit") || text.contains("updat")),
            ),
            (
                "root-scoped fmt --write",
                text.contains("grund fmt --write") && text.contains("root"),
            ),
            (
                "generated links and section locks",
                text.contains("link") && mentions_locks(&text),
            ),
            ("check verification", text.contains("grund check")),
            (
                "fmt --check verification",
                text.contains("grund fmt --check"),
            ),
            (
                "commit lock changes",
                text.contains("commit") && mentions_locks(&text),
            ),
        ],
    );
}

/// §FS-fmt.6.3, §FS-section-lock.write.rules: the reproducer's existing
/// maintenance succeeds after authored IDs change, with URLs and locks stale.
#[test]
fn authored_rename_already_rederives_links_and_updates_eligible_locks() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    fixture.git(&["init", "-q"]);
    grund(root, &["fmt", "--write"]);
    let old_lock = root.join(".grund/sections/FS-login.lock");
    let old_entry = fs::read_to_string(&old_lock).unwrap();
    assert!(old_entry.starts_with("FS-login.code.1 "));
    fixture.git(&["add", "."]);
    fixture.git(&[
        "-c",
        "user.name=Repro",
        "-c",
        "user.email=repro@example.invalid",
        "commit",
        "-qm",
        "initial fixture",
    ]);
    let old_file = root.join("docs/functional-spec/FS-login.md");
    let new_file = root.join("docs/functional-spec/FS-sign-in.md");
    fs::rename(&old_file, &new_file).unwrap();
    let declaration = fs::read_to_string(&new_file).unwrap();
    fs::write(&new_file, declaration.replace("FS-login", "FS-sign-in")).unwrap();
    for path in ["docs/notes.md", "src/a.py"] {
        let file = root.join(path);
        let text = fs::read_to_string(&file).unwrap();
        fs::write(file, text.replace("\u{a7}FS-login", "\u{a7}FS-sign-in")).unwrap();
    }
    let notes = root.join("docs/notes.md");
    let stale = fs::read_to_string(&notes).unwrap();
    assert!(
        stale.contains("[\u{a7}FS-sign-in](functional-spec/FS-login.md#fs-login-a-user-signs-in)")
    );
    assert_eq!(fs::read_to_string(&old_lock).unwrap(), old_entry);
    grund(root, &["fmt", "--write"]);
    assert_eq!(
        fs::read_to_string(notes).unwrap(),
        "See [\u{a7}FS-sign-in](functional-spec/FS-sign-in.md#fs-sign-in-a-user-signs-in) \
         and [\u{a7}FS-sign-in.code.1](functional-spec/FS-sign-in.md#1-lifetime).\n"
    );
    assert!(!old_lock.exists());
    assert_eq!(
        fs::read_to_string(root.join(".grund/sections/FS-sign-in.lock")).unwrap(),
        old_entry.replace("FS-login", "FS-sign-in")
    );
    grund(root, &["check"]);
    grund(root, &["fmt", "--check"]);
}
