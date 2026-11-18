//! A byte-order mark that starts a file is part of line 1, so line 1 opens nothing and nothing
//! a block it would open holds declares, in Markdown and in a source file alike
//! (§FS-declarations.line). A declaration on a later line of the same file still declares. The
//! marks are written at run time, so no fixture carries an invisible byte.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\n\n\
                      [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n\
                      [schema.kinds.AR]\nfolders = [\"docs/architecture\"]\nindex = false\n";

const BOM: &str = "\u{feff}";

/// Each file, and whether its declaration is listed.
const FILES: &[(&str, &str, Option<&str>)] = &[
    (
        "docs/functional-spec/FS-login.md",
        "# FS-login: Login\n",
        None,
    ),
    ("src/line.rs", "/// AR-line: A line comment\n", None),
    (
        "src/block.rs",
        "/**\n * AR-block: A block comment\n */\n",
        None,
    ),
    ("src/doc.py", "\"\"\"\nAR-doc: A docstring\n\"\"\"\n", None),
    (
        "src/second.rs",
        "//! A crate comment.\n/// AR-second: A later line\n",
        Some("AR-second"),
    ),
];

struct Tree(PathBuf);

impl Tree {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("grund-bom-decl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/functional-spec")).expect("create the FS folder");
        fs::create_dir_all(root.join("docs/architecture")).expect("create the AR folder");
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        for (path, body, _) in FILES {
            fs::write(root.join(path), format!("{BOM}{body}")).expect("write a marked file");
        }
        Self(root)
    }

    fn run(&self, args: &[&str]) -> (Option<i32>, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
        )
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_byte_order_mark_hides_what_line_one_would_open() {
    let tree = Tree::new();
    let (status, listed) = tree.run(&["list"]);
    assert_eq!(status, Some(0), "{listed}");
    let ids: Vec<&str> = listed
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let expected: Vec<&str> = FILES.iter().filter_map(|(_, _, listed)| *listed).collect();
    assert_eq!(ids, expected, "{listed}");
}
