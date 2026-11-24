//! Black-box: a `markdown link` row of fmt's dry run names every link its line writes, as the text
//! the write replaces, `→`, and the text it writes (§FS-fmt.3.6), so a caller can apply one
//! line's link rewrite by hand. The fixture is grund#1378's own: three links with a missing or a
//! stale anchor and one citation with no link. A second file holds a link whose title carries
//! `, `, which the write drops and an item is read past by its brackets (§FS-fmt.3.6).
//!
//! Each case first proves its expected items against `--write` itself: replacing every left side
//! with its right side in the line on disk gives the line the write leaves (§FS-fmt.3.6). Only
//! then does it compare the preview, `fmt --check` and the no-flag run alike (§FS-fmt.3.5).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"grund-1378\"\n\n[schema]\n\
                      id_format = \"{kind}-{number}-{slug}\"\n\n[schema.kinds.SRC]\n\
                      folders = [\"docs/sources\"]\nindex = false\n";

const README: &str = "**Sources**\n\n| Source | Summary |\n|---|---|\n\
                      | [\u{a7}SRC-501-alpha](SRC-501-alpha.md) | link, no anchor |\n\
                      | [\u{a7}SRC-502-beta](SRC-502-beta.md) | link, no anchor |\n\
                      | [\u{a7}SRC-503-gamma](SRC-503-gamma.md#stale) | link, stale anchor |\n\
                      | \u{a7}SRC-504-delta | marked, not linked |\n";

const TITLES: &str = "# Titles\n\nSee [\u{a7}SRC-501-alpha](SRC-501-alpha.md \"Alpha, again\") \
                      and \u{a7}SRC-502-beta.\n";

/// One line's expected row: its number, and each item as (replaced, written), in source order.
struct Row {
    line: usize,
    items: &'static [(&'static str, &'static str)],
}

impl Row {
    fn render(&self, path: &str) -> String {
        let items: Vec<String> = self
            .items
            .iter()
            .map(|(old, new)| format!("{old} \u{2192} {new}"))
            .collect();
        format!("{path}:{}: markdown link: {}", self.line, items.join(", "))
    }
}

const README_ROWS: &[Row] = &[
    Row {
        line: 5,
        items: &[(
            "[\u{a7}SRC-501-alpha](SRC-501-alpha.md)",
            "[\u{a7}SRC-501-alpha](SRC-501-alpha.md#src-501-alpha-alpha-source-report)",
        )],
    },
    Row {
        line: 6,
        items: &[(
            "[\u{a7}SRC-502-beta](SRC-502-beta.md)",
            "[\u{a7}SRC-502-beta](SRC-502-beta.md#src-502-beta-beta-source-report)",
        )],
    },
    Row {
        line: 7,
        items: &[(
            "[\u{a7}SRC-503-gamma](SRC-503-gamma.md#stale)",
            "[\u{a7}SRC-503-gamma](SRC-503-gamma.md#src-503-gamma-gamma-source-report)",
        )],
    },
    Row {
        line: 8,
        items: &[(
            "\u{a7}SRC-504-delta",
            "[\u{a7}SRC-504-delta](SRC-504-delta.md#src-504-delta-delta-source-report)",
        )],
    },
];

const TITLE_ROWS: &[Row] = &[Row {
    line: 3,
    items: &[
        (
            "[\u{a7}SRC-501-alpha](SRC-501-alpha.md \"Alpha, again\")",
            "[\u{a7}SRC-501-alpha](SRC-501-alpha.md#src-501-alpha-alpha-source-report)",
        ),
        (
            "\u{a7}SRC-502-beta",
            "[\u{a7}SRC-502-beta](SRC-502-beta.md#src-502-beta-beta-source-report)",
        ),
    ],
}];

#[test]
fn the_issues_four_rows_name_the_link_before_and_after() {
    let project = Project::new("readme", "docs/sources/README.md", README);
    project.assert_rows("docs/sources/README.md", README_ROWS);
}

#[test]
fn a_dropped_title_holding_a_comma_is_one_item() {
    let project = Project::new("titles", "docs/sources/titles.md", TITLES);
    project.assert_rows("docs/sources/titles.md", TITLE_ROWS);
}

/// A version-2 project holding the issue's four `SRC` declarations and one file under test.
struct Project(PathBuf);

impl Project {
    fn new(test: &str, file: &str, content: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1378-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let project = Self(root);
        project.write("grund.toml", CONFIG);
        for (id, title) in [
            ("SRC-501-alpha", "Alpha source report"),
            ("SRC-502-beta", "Beta source report"),
            ("SRC-503-gamma", "Gamma source report"),
            ("SRC-504-delta", "Delta source report"),
        ] {
            project.write(
                &format!("docs/sources/{id}.md"),
                &format!("# {id}: {title}\n\nThe {id} source.\n"),
            );
        }
        project.write(file, content);
        git_init(&project.0);
        let check = project.grund(&project.0, &["check"]);
        assert_eq!(check.0, 0, "the fixture is not clean:\n{}", check.1);
        project
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, content).expect("write the fixture");
    }

    fn grund(&self, dir: &Path, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(dir)
            .output()
            .expect("run grund");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let shown = format!(
            "grund {}: exit {:?}\nstdout:\n{stdout}\nstderr:\n{}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        (output.status.code().expect("exited with a status"), shown)
    }

    /// The expected items are what `--write` does (§FS-fmt.3.6), and the preview names them.
    fn assert_rows(&self, file: &str, rows: &[Row]) {
        let before = fs::read_to_string(self.0.join(file)).expect("read the file");
        let scratch = self.0.with_extension("write");
        let _ = fs::remove_dir_all(&scratch);
        copy_tree(&self.0, &scratch);
        let (code, shown) = self.grund(&scratch, &["fmt", "--write", file]);
        assert_eq!(code, 0, "{shown}");
        let after = fs::read_to_string(scratch.join(file)).expect("read the written file");
        let _ = fs::remove_dir_all(&scratch);
        for row in rows {
            let mut line = before
                .lines()
                .nth(row.line - 1)
                .expect("the line")
                .to_owned();
            let mut from = 0;
            for (old, new) in row.items {
                let at = from + line[from..].find(old).expect("the span stands on the line");
                line.replace_range(at..at + old.len(), new);
                from = at + new.len();
            }
            assert_eq!(
                line,
                after.lines().nth(row.line - 1).expect("the written line"),
                "line {}: the expected items do not give the line --write leaves",
                row.line
            );
        }

        let expected: String = rows.iter().map(|row| row.render(file) + "\n").collect();
        for args in [&["fmt", "--check", file][..], &["fmt", file][..]] {
            let (code, shown) = self.grund(&self.0, args);
            assert_eq!(code, 1, "{shown}");
            let stdout = shown
                .split_once("stdout:\n")
                .and_then(|(_, rest)| rest.split_once("\nstderr:\n"))
                .map(|(out, _)| out)
                .expect("the run's stdout");
            assert_eq!(stdout, expected, "{shown}");
        }
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create the copy");
    for entry in fs::read_dir(from).expect("read the tree") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a file type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy a file");
        }
    }
}

fn git_init(dir: &Path) {
    let status = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(dir)
        .status()
        .expect("run git init");
    assert!(status.success(), "git init failed");
}
