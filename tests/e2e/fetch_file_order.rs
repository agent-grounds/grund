//! File-home insertion order follows the effective format (§FS-fetch.4.order).
//! These CLI scenarios pin #1361 without changing list order or replacement ownership.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Tree(PathBuf);

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Tree {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1361-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("claim fixture directory");
        Self(root)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn project(&self, dir: &str, format: &str, per_kind: bool) {
        let setting = format!("id_format = \"{format}\"\n");
        self.write(
            &format!("{dir}/grund.toml"),
            &format!(
                "grund_config_version = 2\nproject_name = \"graal\"\n\n\
                 [schema]\nslug_pattern = \"[a-z][a-z0-9-]*\"\n{}\n\
                 [schema.kinds.GR]\nfiles = [\"docs/tickets.md\"]\n{}\
                 fetch = \"scripts/fetch\"\n",
                if per_kind { "" } else { &setting },
                if per_kind { &setting } else { "" },
            ),
        );
        let script = format!("{dir}/scripts/fetch");
        self.write(
            &script,
            "#!/bin/sh\nprintf '## %s: Ticket %s\\n\\nFetched body.\\n' \"$1\" \"$1\"\n",
        );
        fs::set_permissions(self.0.join(script), fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn run(&self, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("run grund");
        assert!(
            output.status.success(),
            "{args:?}: status {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }

    fn file_ids(&self, path: &str) -> Vec<String> {
        self.read(path)
            .lines()
            .filter_map(|line| line.strip_prefix("## ")?.split_once(':'))
            .map(|(id, _)| id.to_owned())
            .collect()
    }

    fn assert_numeric_order(&self, home: &str, prefix: &str, expected: &[&str]) {
        let listed: Vec<String> = self
            .run(&["list", "--format", "json"])
            .lines()
            .map(|line| {
                let row: serde_json::Value = serde_json::from_str(line).unwrap();
                row["id"]
                    .as_str()
                    .unwrap()
                    .strip_prefix(prefix)
                    .unwrap()
                    .to_owned()
            })
            .collect();
        let file = self.file_ids(home);
        println!("file order: {file:?}\nlist order: {listed:?}\nexpected: {expected:?}");
        assert_eq!(listed, expected, "list must already compare numeric IDs");
        assert_eq!(file, expected, "file-home insertion must agree with list");
    }
}

/// §FS-fetch.4.order: unpadded per-kind numbers in a version-2 workspace member.
#[test]
#[ignore = "red until #1361 compares the number numerically; implement removes this"]
fn workspace_member_inserts_unpadded_numbers_in_numeric_order() {
    let tree = Tree::new();
    tree.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"workspace\"\n\n\
         [workspace]\nmembers = [\"graal\"]\ninclude_root = false\n",
    );
    tree.project("graal", "{kind}-{number}", true);
    assert!(!tree.0.join("graal/docs/tickets.md").exists());
    for id in ["GR-9", "GR-80923", "GR-1000", "GR-100"] {
        tree.run(&["fetch", &format!("graal/{id}")]);
    }
    tree.assert_numeric_order(
        "graal/docs/tickets.md",
        "graal/",
        &["GR-9", "GR-100", "GR-1000", "GR-80923"],
    );
}

/// §FS-fetch.4.order: inherited project format still compares numerically past width three.
#[test]
#[ignore = "red until #1361 compares the number numerically; implement removes this"]
fn project_format_inserts_numbers_across_the_padding_boundary() {
    let tree = Tree::new();
    tree.project(".", "{kind}-{number}", false);
    assert!(!tree.0.join("docs/tickets.md").exists());
    for id in ["GR-1000", "GR-999", "GR-9", "GR-12"] {
        tree.run(&["fetch", id]);
    }
    tree.assert_numeric_order(
        "docs/tickets.md",
        "",
        &["GR-9", "GR-12", "GR-999", "GR-1000"],
    );
}

/// §FS-fetch.4.order: number-before-slug precedence, with lexical slug tie-breaking.
#[test]
#[ignore = "red until #1361 compares the number numerically; implement removes this"]
fn number_before_slug_uses_numeric_precedence_then_lexical_slug() {
    let tree = Tree::new();
    tree.project(".", "{kind}-{number}-{slug}", true);
    for id in ["GR-1000-alpha", "GR-9-zeta", "GR-999-alpha", "GR-9-alpha"] {
        tree.run(&["fetch", id]);
    }
    assert_eq!(
        tree.file_ids("docs/tickets.md"),
        ["GR-9-alpha", "GR-9-zeta", "GR-999-alpha", "GR-1000-alpha"]
    );
}

/// §FS-fetch.4.order: digits inside a slug stay lexical; a generic natural sort is wrong.
/// This guard passes before #1361; the numeric scenarios above are the red regressions.
#[test]
fn slug_digits_stay_lexical_before_the_number() {
    let tree = Tree::new();
    tree.project(".", "{kind}-{slug}-{number}", true);
    for id in ["GR-item9-1", "GR-item10-9", "GR-alpha-9", "GR-zeta-1"] {
        tree.run(&["fetch", id]);
    }
    assert_eq!(
        tree.file_ids("docs/tickets.md"),
        ["GR-alpha-9", "GR-item10-9", "GR-item9-1", "GR-zeta-1"]
    );
}

/// §FS-fetch.4.order: a tied slug reaches numeric comparison in a slug-first format too.
#[test]
#[ignore = "red until #1361 compares the number numerically; implement removes this"]
fn slug_before_number_compares_tied_slugs_by_numeric_value() {
    let tree = Tree::new();
    tree.project(".", "{kind}-{slug}-{number}", true);
    for id in ["GR-zeta-1", "GR-alpha-100", "GR-alpha-9"] {
        tree.run(&["fetch", id]);
    }
    assert_eq!(
        tree.file_ids("docs/tickets.md"),
        ["GR-alpha-9", "GR-alpha-100", "GR-zeta-1"]
    );
}

/// §FS-fetch.4: replacing an existing declaration keeps its position and surrounding bytes.
/// This guard passes before #1361; insertion must not become a whole-file sort.
#[test]
fn refetch_keeps_position_in_an_unsorted_home() {
    let tree = Tree::new();
    tree.project(".", "{kind}-{number}", true);
    let before = "# Tickets\n\nPreamble.\n\n## GR-1000: First\n\nUnchanged first.\n\n\
                  ## GR-9: Old\n\nOld body.\n\n\n## GR-12: Last\n\nUnchanged last.\n";
    tree.write("docs/tickets.md", before);
    tree.run(&["fetch", "GR-9"]);
    let expected = before.replace(
        "## GR-9: Old\n\nOld body.",
        "## GR-9: Ticket GR-9\n\nFetched body.",
    );
    assert_eq!(tree.read("docs/tickets.md"), expected);
    tree.run(&["fetch", "GR-9"]);
    assert_eq!(tree.read("docs/tickets.md"), expected);
}
