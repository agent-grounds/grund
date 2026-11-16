//! A version-1 project's multi-line string literals and quotes read the same after
//! `grund config migrate --to 2` as before it (§FS-languages.multi-line-string.suffix,
//! §FS-config-v2.mapping). The tree holds a Rust lifetime before a trailing comment and a Rust raw
//! string, a TypeScript template literal, a Python `call("""` string, a Go raw string, a Java text
//! block and a shell heredoc, each with a qualified citation of the unconfigured alias `other`
//! inside it. Version 1 reads each interior as a string, so the one line it reports is the real
//! comment after `'static`. Migration must say the records are equal on this tree and write the
//! file it writes today (§FS-config-migrate.5.1, §FS-config-migrate.5.4), and the migrated config
//! must report what version 1 reports. Issue #1406 is the report: on a tree without the literals
//! migration said the records are equal and wrote the file, and the migrated config then reported
//! every interior and hid the citation after `'static`.
//!
//! These are not goldens because a golden has no skip, and every test here but the version-1
//! control is red until a `source` definition's files read their own suffix's row and quote
//! reading. The trees are built in the temp directory, so run them with a `TMPDIR` that no git
//! tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The version-1 config: the proposal's suffixes, Java and shell, and every other key at its
/// default.
const VERSION_1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
                         named_sections = true\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
                         index = false\n[scan]\ninclude = [\"docs\", \"src\"]\n\
                         extensions = [\"md\", \"rs\", \"ts\", \"py\", \"go\", \"java\", \"sh\"]\n";

/// What `grund config migrate --to 2 --write` writes for [`VERSION_1`], byte for byte, as grund
/// wrote it before #1406: every suffix a `v1_*` definition. The fix changes what it means, not
/// its bytes (§FS-config-v2.schema.sources.migration).
const MIGRATED: &str = "\
    grund_config_version = 2\n\
    \n\
    [schema]\n\
    marker = \"\u{a7}\"\n\
    shorthand = \"canonical\"\n\
    id_format = \"{kind}-{slug}\"\n\
    slug_pattern = \"[a-z0-9][a-z0-9-]*\"\n\
    number_pattern = \"\\\\d+\"\n\
    section_separator = \".\"\n\
    heading_depth = \"must\"\n\
    \n\
    [schema.sources]\n\
    languages = [\"v1_markdown\", \"v1_c_family\", \"v1_python\", \"v1_go\", \"v1_shell\"]\n\
    declarations = \"any-comment\"\n\
    exclude = [\"target/\", \"node_modules/\", \".git/\", \"dist/\", \"build/\", \".venv/\"]\n\
    respect_gitignore = true\n\
    fenced = []\n\
    \n\
    [schema.sources.extensions]\n\
    md = \"v1_markdown\"\n\
    rs = \"v1_c_family\"\n\
    ts = \"v1_c_family\"\n\
    py = \"v1_python\"\n\
    go = \"v1_go\"\n\
    java = \"v1_c_family\"\n\
    sh = \"v1_shell\"\n\
    \n\
    [schema.sources.definitions.v1_markdown]\n\
    class = \"document\"\n\
    comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n\
    doc_comments = \"none\"\n\
    docstrings = \"off\"\n\
    \n\
    [schema.sources.definitions.v1_c_family]\n\
    class = \"source\"\n\
    comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n\
    doc_comments = \"c-family\"\n\
    docstrings = \"off\"\n\
    \n\
    [schema.sources.definitions.v1_python]\n\
    class = \"source\"\n\
    comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n\
    doc_comments = \"python\"\n\
    docstrings = \"python\"\n\
    \n\
    [schema.sources.definitions.v1_go]\n\
    class = \"source\"\n\
    comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n\
    doc_comments = \"go\"\n\
    docstrings = \"off\"\n\
    \n\
    [schema.sources.definitions.v1_shell]\n\
    class = \"source\"\n\
    comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n\
    doc_comments = \"shell\"\n\
    docstrings = \"off\"\n\
    \n\
    [schema.notes.lines]\n\
    must = 3\n\
    may = 1\n\
    \n\
    [schema.notes.columns]\n\
    must = 100\n\
    \n\
    [schema.notes.text]\n\
    must = true\n\
    \n\
    [schema.kinds.FS]\n\
    folders = [\"docs\"]\n\
    index = false\n\
    \n\
    [schema.kinds.code]\n\
    folders = [\"src\"]\n\
    \n\
    [presentation]\n\
    trigger = \"$$\"\n\
    rules = \"root\"\n\
    \n\
    [presentation.fmt]\n\
    anchors = \"github\"\n\
    links = \"index\"\n\
    exclude = []\n";

/// Each file under `src/`: a citation of `FS-login` on line 1, and the qualified citation of
/// `other` inside a literal, or after `'static` on `lib.rs` line 2.
const SOURCES: &[(&str, &str)] = &[
    (
        "lib.rs",
        "// \u{a7}FS-login\nfn f(x: &'static str) {} // \u{a7}other/FS-after-lifetime\n\
         const S: &str = r#\"\n// \u{a7}other/FS-in-raw\n\"#;\n",
    ),
    (
        "a.ts",
        "// \u{a7}FS-login\nconst t = `\n// \u{a7}other/FS-in-template\n`;\n",
    ),
    (
        "a.py",
        "# \u{a7}FS-login\ncall(\"\"\"\n# \u{a7}other/FS-in-triple\n\"\"\")\n",
    ),
    (
        "a.go",
        "// \u{a7}FS-login\nvar s = `\n// \u{a7}other/FS-in-raw\n`\n",
    ),
    (
        "Text.java",
        "// \u{a7}FS-login\nclass Text { String s = \"\"\"\n// \u{a7}other/FS-in-string\n\"\"\"; }\n",
    ),
    (
        "run.sh",
        "# \u{a7}FS-login\ncat <<EOF\n# \u{a7}other/FS-in-heredoc\nEOF\n",
    ),
];

/// What version 1 reports on the tree: the comment after `'static`, and nothing inside a literal.
const VERSION_1_REPORT: &str = "src/lib.rs:2: error: unknown project alias other\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// `config` as `grund.toml`, `docs/FS-login.md`, and every file of [`SOURCES`].
    fn new(name: &str, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1406-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the tree");
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("grund.toml"), config).expect("write the config");
        fs::write(
            root.join("docs/FS-login.md"),
            "# FS-login: Login\n\nText.\n",
        )
        .expect("write the spec");
        for (path, text) in SOURCES {
            fs::write(root.join("src").join(path), text).expect("write a source");
        }
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
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

/// §FS-languages.multi-line-string.suffix: the control. Version 1 reads every literal's interior
/// as a string and the citation after `'static` as a comment. This passes before the fix.
#[test]
fn check_version_1_multi_line_forms() {
    let tree = Tree::new("version-1", VERSION_1);
    let output = tree.run(&["check", "--suggestions"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        VERSION_1_REPORT,
        "{}",
        streams(&output)
    );
    assert_eq!(output.status.code(), Some(1), "{}", streams(&output));
}

/// §FS-config-migrate.5.1, §FS-config-migrate.5.4, §FS-config-v2.mapping: migrating the tree,
/// literals and all, says the records are equal and the report on this tree is the same, exits
/// `0`, and previews the file [`MIGRATED`] holds. Today the report gate refuses: the candidate
/// reports every interior and misses `src/lib.rs:2`.
#[test]
fn config_migrate_keeps_multi_line_forms() {
    let tree = Tree::new("migrate", VERSION_1);
    let output = tree.run(&["config", "migrate", "--to", "2"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "\ncompared: the records are equal, and grund check --suggestions on this tree gives \
             the same findings and the same exit before and after\n"
        ) && !stderr.contains("differs:"),
        "{}",
        streams(&output)
    );
    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
    assert_eq!(String::from_utf8_lossy(&output.stdout), MIGRATED);
}

/// §FS-languages.multi-line-string.suffix, §FS-config-v2.schema.sources.definitions: the same
/// tree under the config migration writes reports what version 1 reports, and nothing else.
/// Today it reports the six interiors and misses `src/lib.rs:2`.
#[test]
fn check_migrated_multi_line_forms() {
    let tree = Tree::new("migrated", MIGRATED);
    let output = tree.run(&["check", "--suggestions"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        VERSION_1_REPORT,
        "{}",
        streams(&output)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(1));
}
