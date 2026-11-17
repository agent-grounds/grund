//! agent-grounds/grund#1384, end to end. Part 2 is `grund config adopt-builtins`, run on the
//! issue's project as `grund config migrate --to 2 --write` leaves it
//! (§FS-config-adopt-builtins). Part 1 is migration naming a built-in row where it reads a
//! suffix exactly as version 1 did (§FS-config-v2.schema.sources.migration).
//!
//! These are not goldens, because a golden has no skip. Every test marked `#[ignore]` is red
//! until the fix; the others are guards that hold before it and must hold after. The trees are
//! built in the temp directory, so run them with a `TMPDIR` that no git tree or grund config
//! encloses.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The issue's version-1 config.
const VERSION_1: &str = concat!(
    "grund_config_version = 1\n",
    "[id]\n",
    "format = \"{kind}-{slug}\"\n",
    "named_sections = true\n",
    "[[kinds]]\n",
    "kind = \"FS\"\n",
    "folder = \"docs\"\n",
    "index = false\n",
    "[scan]\n",
    "include = [\"docs\", \"src\"]\n",
    "extensions = [\"md\", \"java\", \"gradle\", \"sh\"]\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
);

/// What migration writes for [`VERSION_1`], up to `languages`.
const HEAD: &str = concat!(
    "grund_config_version = 2\n",
    "\n",
    "[schema]\n",
    "marker = \"\u{a7}\"\n",
    "shorthand = \"canonical\"\n",
    "id_format = \"{kind}-{slug}\"\n",
    "slug_pattern = \"[a-z0-9][a-z0-9-]*\"\n",
    "number_pattern = \"\\\\d+\"\n",
    "section_separator = \".\"\n",
    "heading_depth = \"must\"\n",
    "\n",
    "[schema.sources]\n",
);

/// The rest of `[schema.sources]`, and the mapping table's header.
const SOURCE_KEYS: &str = concat!(
    "declarations = \"any-comment\"\n",
    "exclude = [\"target/\", \"node_modules/\", \".git/\", \"dist/\", \"build/\", \".venv/\"]\n",
    "respect_gitignore = true\n",
    "fenced = []\n",
    "\n",
    "[schema.sources.extensions]\n",
);

const MAPPINGS: &str = concat!(
    "md = \"v1_markdown\"\n",
    "java = \"v1_c_family\"\n",
    "gradle = \"v1_plain\"\n",
    "sh = \"v1_shell\"\n",
    "\n",
);

const DEFINITIONS: &str = concat!(
    "[schema.sources.definitions.v1_markdown]\n",
    "class = \"document\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"none\"\n",
    "docstrings = \"off\"\n",
    "\n",
    "[schema.sources.definitions.v1_c_family]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"c-family\"\n",
    "docstrings = \"off\"\n",
    "\n",
);

/// The one definition adoption keeps, for `.gradle`.
const PLAIN: &str = concat!(
    "[schema.sources.definitions.v1_plain]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"none\"\n",
    "docstrings = \"off\"\n",
    "\n",
);

const SHELL: &str = concat!(
    "[schema.sources.definitions.v1_shell]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"shell\"\n",
    "docstrings = \"off\"\n",
    "\n",
);

const TAIL: &str = concat!(
    "[schema.notes.lines]\n",
    "must = 3\n",
    "may = 1\n",
    "\n",
    "[schema.notes.columns]\n",
    "must = 100\n",
    "\n",
    "[schema.notes.text]\n",
    "must = true\n",
    "\n",
    "[schema.kinds.FS]\n",
    "folders = [\"docs\"]\n",
    "index = false\n",
    "\n",
    "[schema.kinds.code]\n",
    "folders = [\"src\"]\n",
    "\n",
    "[presentation]\n",
    "trigger = \"$$\"\n",
    "rules = \"root\"\n",
    "\n",
    "[presentation.fmt]\n",
    "anchors = \"github\"\n",
    "links = \"index\"\n",
    "exclude = []\n",
);

/// What `grund config migrate --to 2 --write` writes for [`VERSION_1`] today, byte for byte. Part
/// 1 leaves it so: no suffix here reads as its built-in row does.
fn migrated() -> String {
    let languages = "languages = [\"v1_markdown\", \"v1_c_family\", \"v1_plain\", \"v1_shell\"]\n";
    format!("{HEAD}{languages}{SOURCE_KEYS}{MAPPINGS}{DEFINITIONS}{PLAIN}{SHELL}{TAIL}")
}

/// [`migrated`] after adoption: `languages` renamed in place, the adopted mappings and the
/// definitions nothing names removed, and every other byte kept (§FS-config-adopt-builtins.4).
fn candidate() -> String {
    let languages = "languages = [\"markdown\", \"java\", \"v1_plain\", \"shell\"]\n";
    format!("{HEAD}{languages}{SOURCE_KEYS}gradle = \"v1_plain\"\n\n{PLAIN}{TAIL}")
}

/// [`migrated`] with a comment on the `java` mapping, which adoption removes; it is line 21.
fn migrated_with_comment() -> String {
    migrated().replacen(
        "java = \"v1_c_family\"\n",
        "# Java sources\njava = \"v1_c_family\"\n",
        1,
    )
}

/// The issue's tree, with a Java text block, a `.bash` file `shell` admits, and a `.py` file the
/// explicit list never admits.
const ISSUE_FILES: &[(&str, &str)] = &[
    ("docs/FS-login.md", "# FS-login: Login\n\nUsers log in.\n"),
    ("src/Login.java", "// \u{a7}FS-login\nclass Login {}\n"),
    (
        "src/build.gradle",
        "// \u{a7}FS-login\n// \u{a7}FS-typo\napply plugin: 'java'\n",
    ),
    ("src/run.sh", "# \u{a7}FS-login\necho hi\n"),
    (
        "src/Text.java",
        "class Text { String s = \"\"\"\n// \u{a7}other/FS-in-string\n\"\"\"; }\n",
    ),
    ("src/extra.bash", "echo extra\n# \u{a7}FS-login\n"),
    ("src/tool.py", "# \u{a7}FS-unread\nprint(\"hi\")\n"),
];

const PREVIEW: &str =
    "grund.toml: adopting version 2's built-in languages, preview, nothing written\n";
const WRITE: &str = "grund.toml: adopting version 2's built-in languages, --write\n";

/// The plan's lines on the issue's tree (§FS-config-adopt-builtins.5.2).
const REPORT: &str = concat!(
    "adopts: md: definition v1_markdown \u{2192} built-in markdown\n",
    "  reads differently: comments behind `;` and `--` are now read\n",
    "adopts: java: definition v1_c_family \u{2192} built-in java\n",
    "  reads differently: `#` is no longer a comment host\n",
    "adopts: sh: definition v1_shell \u{2192} built-in shell, opt-in, listed in languages\n",
    "  reads differently: `//`, `/*` and `*` are no longer comment hosts\n",
    "added: bash, zsh: admitted with shell\n",
    "kept: gradle: definition v1_plain and gradle = \"v1_plain\"; \
     version 2 has no built-in language for `.gradle`\n",
);

const COMPARED: &str = "compared: the candidate reads as planned; grund check --suggestions: \
                        exit 1 \u{2192} 1, no finding changes\n";

const WROTE: &str = concat!(
    "wrote grund.toml: adopted built-in markdown, java and shell; kept v1_plain for gradle\n",
    "to get the previous file back: it is in version control, e.g. git restore grund.toml\n",
    "to see this candidate again: restore it and run grund config adopt-builtins\n",
);

const COMMENTS: &str = concat!(
    "comments on lines adoption removes (--write refuses; --drop-comments drops exactly these):\n",
    "  grund.toml:21  above [schema.sources.extensions] java\n",
);

/// A scratch project removed on drop; `name` and a serial keep concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, config: &str, files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("grund-1384-{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a parent")).expect("create a folder");
            fs::write(path, text).expect("write a file");
        }
        fs::write(root.join("grund.toml"), config).expect("write the config");
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_in(&self.0, args)
    }

    fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(dir)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    fn config(&self) -> String {
        fs::read_to_string(self.0.join("grund.toml")).expect("read the config")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The two streams of `output`, for an assertion message.
fn streams(output: &Output) -> String {
    format!("stdout:\n{}\nstderr:\n{}", stdout(output), stderr(output))
}

/// Holds `output` to its three parts.
fn holds(output: &Output, out: &str, err: &str, code: i32) {
    assert_eq!(stderr(output), err, "{}", streams(output));
    assert_eq!(stdout(output), out, "{}", streams(output));
    assert_eq!(output.status.code(), Some(code), "{}", streams(output));
}

/// §FS-config-adopt-builtins.5.1, §FS-config-adopt-builtins.5.2,
/// §FS-config-adopt-builtins.5.3, §FS-config-adopt-builtins.6.4: the preview prints the candidate
/// on stdout and the plan on stderr, exits `0`, and writes nothing.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_previews_the_candidate_and_every_reading_that_changes() {
    let tree = Tree::new("preview", &migrated(), ISSUE_FILES);
    let output = tree.run(&["config", "adopt-builtins"]);
    holds(
        &output,
        &candidate(),
        &format!("{PREVIEW}{REPORT}{COMPARED}"),
        0,
    );
    assert_eq!(tree.config(), migrated(), "a preview writes nothing");
}

/// §FS-config-adopt-builtins.8.1, §FS-config-adopt-builtins.8.2, §FS-config-adopt-builtins.3.3:
/// the write replaces the file with the candidate, and `check` on the result still reports the
/// citation the issue's hand edit lost, while `src/extra.bash` is now read.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_writes_and_check_still_reports_the_gradle_typo() {
    let tree = Tree::new("write", &migrated(), ISSUE_FILES);
    let output = tree.run(&["config", "adopt-builtins", "--write"]);
    holds(&output, "", &format!("{WRITE}{REPORT}{COMPARED}{WROTE}"), 0);
    assert_eq!(tree.config(), candidate());
    let check = tree.run(&["check"]);
    holds(
        &check,
        "src/build.gradle:2: error: unknown reference FS-typo\n",
        "",
        1,
    );
    let refs = tree.run(&["refs", "FS-login", "--summary"]);
    assert!(
        stdout(&refs).contains("src/extra.bash: 1 (line 2)\n"),
        "{}",
        streams(&refs)
    );
}

/// §FS-config-adopt-builtins.9: on the written file, both a preview and a write print only the
/// kept suffix and the no-op line, exit `0` with empty stdout, and leave the file alone.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_a_second_time_is_a_no_op() {
    let tree = Tree::new("noop", &candidate(), ISSUE_FILES);
    let path = tree.0.join("grund.toml");
    let modified = fs::metadata(&path)
        .and_then(|m| m.modified())
        .expect("mtime");
    let noop = "kept: gradle: definition v1_plain and gradle = \"v1_plain\"; version 2 has no \
                built-in language for `.gradle`\ngrund.toml: nothing to adopt; the file is unchanged\n";
    for args in [
        &["config", "adopt-builtins"][..],
        &["config", "adopt-builtins", "--write"],
    ] {
        holds(&tree.run(args), "", noop, 0);
    }
    assert_eq!(tree.config(), candidate());
    let after = fs::metadata(&path)
        .and_then(|m| m.modified())
        .expect("mtime");
    assert_eq!(after, modified, "the modification time is left alone");
}

/// §FS-config-adopt-builtins.8.1: a config that is a symbolic link is refused, exit `1`.
#[cfg(unix)]
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_refuses_a_symbolic_link() {
    let tree = Tree::new("symlink", &migrated(), ISSUE_FILES);
    fs::rename(tree.0.join("grund.toml"), tree.0.join("real.toml")).expect("move the config");
    std::os::unix::fs::symlink("real.toml", tree.0.join("grund.toml")).expect("link it");
    let output = tree.run(&["config", "adopt-builtins", "--write"]);
    let refused = concat!(
        "refused: grund.toml is a symbolic link, and replacing it would replace the link with a \
         regular file\n",
        "  repair: replace the link with the file it points to, or adopt the built-in languages \
         in that file where it is the config\n",
        "not written: the config is a symbolic link; grund.toml is unchanged\n",
    );
    holds(&output, "", &format!("{WRITE}{refused}"), 1);
    let link = fs::symlink_metadata(tree.0.join("grund.toml")).expect("the link");
    assert!(link.file_type().is_symlink());
    assert_eq!(
        fs::read_to_string(tree.0.join("real.toml")).unwrap(),
        migrated()
    );
}

/// §FS-config-adopt-builtins.7: a comment on a mapping adoption removes refuses the write without
/// `--drop-comments`, exit `1`, and the file is unchanged.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_refuses_to_lose_a_comment() {
    let tree = Tree::new("comment", &migrated_with_comment(), ISSUE_FILES);
    let output = tree.run(&["config", "adopt-builtins", "--write"]);
    let closing = "not written: 1 comment would be lost (pass --drop-comments to drop exactly the \
                   comments listed above); grund.toml is unchanged\n";
    let report = format!("{WRITE}{REPORT}{COMMENTS}{COMPARED}{closing}");
    holds(&output, "", &report, 1);
    assert_eq!(tree.config(), migrated_with_comment());
}

/// §FS-config-adopt-builtins.7, §FS-config-adopt-builtins.8.2: with `--drop-comments` the write
/// drops exactly the listed comment and names it again.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_drops_exactly_the_listed_comment() {
    let tree = Tree::new("drop", &migrated_with_comment(), ISSUE_FILES);
    let output = tree.run(&["config", "adopt-builtins", "--write", "--drop-comments"]);
    let wrote = WROTE.replacen(
        "gradle\n",
        "gradle\ndropped the comments at grund.toml:21 (lines before adoption)\n",
        1,
    );
    holds(
        &output,
        "",
        &format!("{WRITE}{REPORT}{COMMENTS}{COMPARED}{wrote}"),
        0,
    );
    assert_eq!(tree.config(), candidate());
}

/// §FS-config-adopt-builtins.10: a version-1 file is exit `2`, pointing to migration.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_sends_a_version_1_file_to_migration() {
    let tree = Tree::new("version-1", VERSION_1, ISSUE_FILES);
    let output = tree.run(&["config", "adopt-builtins", "--write"]);
    let error = "error: grund.toml is version 1; migrate it first with grund config migrate \
                 --to 2 --write\n";
    holds(&output, "", error, 2);
    assert_eq!(tree.config(), VERSION_1);
}

/// §FS-config-adopt-builtins.1.2: `--to` and `--format` are unknown flags, exit `2`.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_refuses_to_and_format() {
    let tree = Tree::new("flags", &migrated(), ISSUE_FILES);
    let to = tree.run(&["config", "adopt-builtins", "--to", "3"]);
    holds(&to, "", "error: unknown flag `--to`\n", 2);
    let format = tree.run(&["config", "adopt-builtins", "--format", "json"]);
    holds(&format, "", "error: unknown flag `--format`\n", 2);
    assert_eq!(tree.config(), migrated());
}

/// §FS-config-adopt-builtins.1.1, §FS-config-adopt-builtins.1.2: `--path-base invocation` from
/// `src/` spells the config from there.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_spells_paths_from_the_invocation() {
    let tree = Tree::new("path-base", &migrated(), ISSUE_FILES);
    let output = tree.run_in(
        &tree.0.join("src"),
        &["config", "adopt-builtins", "--path-base", "invocation"],
    );
    let heading = format!("../{PREVIEW}");
    holds(
        &output,
        &candidate(),
        &format!("{heading}{REPORT}{COMPARED}"),
        0,
    );
}

/// A version-2 project with no `code` row: `docs/FS-login.md` and one source file.
fn small(source: &'static str, text: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "docs/FS-login.md",
            "# FS-login: Login\n\nUsers log in.\n".into(),
        ),
        (source, text.to_string()),
    ]
}

fn borrowed<'a>(files: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    files.iter().map(|(p, t)| (*p, t.as_str())).collect()
}

/// A `v1_markdown` definition that reads `.md` exactly as built-in `markdown` does.
const MARKDOWN_DEFINITION: &str = concat!(
    "[schema.sources.definitions.v1_markdown]\n",
    "class = \"document\"\n",
    "comment_prefixes = [\"//\", \"#\", \";\", \"--\", \"*\", \"/*\"]\n",
    "\n",
);

const FS_ROW: &str = "[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";

/// A clean tree's report, after its `adopts:` and `kept:` lines.
const CLEAN: &str = "compared: the candidate reads as planned; grund check --suggestions: \
                     exit 0 \u{2192} 0, no finding changes\n";

const ADOPTS_MD: &str = "adopts: md: definition v1_markdown \u{2192} built-in markdown\n";

/// §FS-config-adopt-builtins.3.2, §FS-config-adopt-builtins.3.1: `.json` read through a
/// `source` definition is kept, because built-in `json` reads it as `data`; `.md` is still
/// adopted, with no `reads differently:` line since it reads the same.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_keeps_a_suffix_its_row_would_reclassify() {
    let notes = "[schema.sources.definitions.notes]\nclass = \"source\"\n\
                 comment_prefixes = [\"//\"]\n\n";
    let config = format!(
        "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"v1_markdown\", \"notes\"]\n\n\
         [schema.sources.extensions]\nmd = \"v1_markdown\"\njson = \"notes\"\n\n\
         {MARKDOWN_DEFINITION}{notes}{FS_ROW}"
    );
    let adopted = format!(
        "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"markdown\", \"notes\"]\n\n\
         [schema.sources.extensions]\njson = \"notes\"\n\n{notes}{FS_ROW}"
    );
    let files = small("src/a.json", "// \u{a7}FS-login\n{}\n");
    let tree = Tree::new("json", &config, &borrowed(&files));
    let output = tree.run(&["config", "adopt-builtins"]);
    let kept = "kept: json: built-in json reads it as data\n";
    holds(
        &output,
        &adopted,
        &format!("{PREVIEW}{ADOPTS_MD}{kept}{CLEAN}"),
        0,
    );
}

/// §FS-config-adopt-builtins.2, §FS-config-adopt-builtins.4.1, §FS-config-adopt-builtins.4.2:
/// `kt = "java"` maps to a built-in already, so it is not in play and stays as it is.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_leaves_a_suffix_already_mapped_to_a_builtin() {
    let config = format!(
        "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"v1_markdown\", \"java\"]\n\n\
         [schema.sources.extensions]\nmd = \"v1_markdown\"\nkt = \"java\"\n\n\
         {MARKDOWN_DEFINITION}{FS_ROW}"
    );
    let adopted = format!(
        "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"markdown\", \"java\"]\n\n\
         [schema.sources.extensions]\nkt = \"java\"\n\n{FS_ROW}"
    );
    let files = small("src/A.kt", "// \u{a7}FS-login\nclass A\n");
    let tree = Tree::new("kt", &config, &borrowed(&files));
    let output = tree.run(&["config", "adopt-builtins"]);
    holds(
        &output,
        &adopted,
        &format!("{PREVIEW}{ADOPTS_MD}{CLEAN}"),
        0,
    );
}

/// §FS-config-adopt-builtins.2, §FS-config-adopt-builtins.9: with `languages` omitted no
/// definition is admitted, so there is nothing to adopt, preview or write.
#[test]
#[ignore = "red until #1384 adds grund config adopt-builtins; implement removes this"]
fn adopt_has_nothing_to_adopt_when_languages_is_omitted() {
    let config = format!(
        "grund_config_version = 2\n\n[schema.sources.definitions.v1_plain]\nclass = \"source\"\n\
         comment_prefixes = [\"//\"]\n\n{FS_ROW}"
    );
    let files = small("src/A.java", "// \u{a7}FS-login\nclass A {}\n");
    let tree = Tree::new("omitted", &config, &borrowed(&files));
    for args in [
        &["config", "adopt-builtins"][..],
        &["config", "adopt-builtins", "--write"],
    ] {
        holds(
            &tree.run(args),
            "",
            "grund.toml: nothing to adopt; the file is unchanged\n",
            0,
        );
    }
    assert_eq!(tree.config(), config);
}

/// A version-1 file whose only suffix is `md`, read through the default prefixes.
const MARKDOWN_ONLY: &str = concat!(
    "grund_config_version = 1\n",
    "[id]\n",
    "format = \"{kind}-{slug}\"\n",
    "named_sections = true\n",
    "[[kinds]]\n",
    "kind = \"FS\"\n",
    "folder = \"docs\"\n",
    "index = false\n",
    "[scan]\n",
    "include = [\"docs\"]\n",
    "extensions = [\"md\"]\n",
);

/// §FS-config-v2.schema.sources.migration, §FS-config-migrate.3.3, §FS-config-migrate.mapping:
/// the default prefixes are built-in `markdown`'s hosts, so migration names `markdown` and writes
/// no mapping and no definition. Today it writes `v1_markdown`.
#[test]
#[ignore = "red until #1384 names a built-in where it reads as version 1; implement removes this"]
fn migrate_names_builtin_markdown_for_the_default_prefixes() {
    let docs = [("docs/FS-login.md", "# FS-login: Login\n\nUsers log in.\n")];
    let tree = Tree::new("markdown-only", MARKDOWN_ONLY, &docs);
    let output = tree.run(&["config", "migrate", "--to", "2"]);
    let candidate = format!(
        "{HEAD}languages = [\"markdown\"]\n{}{}",
        SOURCE_KEYS.replace("\n[schema.sources.extensions]\n", ""),
        "\n[schema.notes.lines]\nmust = 3\nmay = 1\n\n[schema.notes.columns]\nmust = 100\n\n\
         [schema.notes.text]\nmust = true\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\n\
         [presentation]\ntrigger = \"$$\"\nrules = \"root\"\n\n[presentation.fmt]\n\
         anchors = \"github\"\nlinks = \"index\"\nexclude = []\n",
    );
    let written = |line: &str| format!("written: {line}: version 1's default, written out\n");
    let report = [
        "grund.toml: version 1 \u{2192} 2, preview, nothing written\n".to_string(),
        "mapped: grund.toml:1: grund_config_version \u{2192} grund_config_version = 2\n".into(),
        "mapped: grund.toml:3: [id] format \u{2192} [schema] id_format\n".into(),
        "mapped: grund.toml:4: [id] named_sections \u{2192} nothing, version 2 always means it\n"
            .into(),
        "mapped: grund.toml:6: [[kinds]] kind \u{2192} [schema.kinds.FS]\n".into(),
        "mapped: grund.toml:7: [[kinds]] folder \u{2192} [schema.kinds.FS] folders\n".into(),
        "mapped: grund.toml:8: [[kinds]] index \u{2192} [schema.kinds.FS] index\n".into(),
        "mapped: grund.toml:10: [scan] include \u{2192} nothing, docs is a place of \
         [schema.kinds.FS] already\n"
            .into(),
        "mapped: grund.toml:11: [scan] extensions \u{2192} [schema.sources] languages, \
         built-in markdown\n"
            .into(),
        written("[schema] slug_pattern = \"[a-z0-9][a-z0-9-]*\""),
        written("[schema.sources] declarations = \"any-comment\""),
        written(
            "[schema.sources] exclude = [\"target/\", \"node_modules/\", \".git/\", \"dist/\", \
             \"build/\", \".venv/\"]",
        ),
        written("[schema.notes.lines] may = 1"),
        written("[presentation] rules = \"root\""),
        "compared: the records are equal, and grund check --suggestions on this tree gives the \
         same findings and the same exit before and after\n"
            .into(),
    ]
    .concat();
    holds(&output, &candidate, &report, 0);
}

/// §FS-config-v2.schema.sources.migration: on the issue's project no suffix reads as its row
/// does, so migration writes the `v1_*` definitions it writes today. Holds before and after.
#[test]
fn migrate_keeps_the_issue_project_on_its_definitions() {
    let tree = Tree::new("issue-migrate", VERSION_1, &ISSUE_FILES[..4]);
    let output = tree.run(&["config", "migrate", "--to", "2"]);
    assert_eq!(stdout(&output), migrated(), "{}", streams(&output));
    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
}

/// §FS-config-migrate.5.1: comment hosts are matched longest first, so a definition over `.java`
/// reads the same whatever order its hosts are listed in, and the same as built-in `java`. Holds
/// before and after: the records gate may compare hosts as a set because this does.
#[test]
fn host_order_never_changes_what_a_file_reads() {
    let source = "// \u{a7}FS-login\n// \u{a7}FS-slash\n/* \u{a7}FS-block */\n/**\n \
                  * \u{a7}FS-star\n */\nclass A {}\n";
    let files = small("src/A.java", source);
    let builtin = format!(
        "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"markdown\", \"java\"]\n\n\
         {FS_ROW}"
    );
    let tree = Tree::new("hosts", &builtin, &borrowed(&files));
    let expected = tree.run(&["check", "--suggestions"]);
    assert_eq!(expected.status.code(), Some(1), "{}", streams(&expected));
    assert_eq!(
        stdout(&expected).lines().count(),
        3,
        "{}",
        streams(&expected)
    );
    let orders = [
        "\"//\", \"/*\", \"*\"",
        "\"//\", \"*\", \"/*\"",
        "\"/*\", \"//\", \"*\"",
        "\"/*\", \"*\", \"//\"",
        "\"*\", \"//\", \"/*\"",
        "\"*\", \"/*\", \"//\"",
    ];
    for hosts in orders {
        let config = format!(
            "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"markdown\", \"perm\"]\n\n\
             [schema.sources.extensions]\njava = \"perm\"\n\n\
             [schema.sources.definitions.perm]\nclass = \"source\"\ncomment_prefixes = [{hosts}]\n\
             doc_comments = \"c-family\"\n\n{FS_ROW}"
        );
        fs::write(tree.0.join("grund.toml"), &config).expect("write the config");
        let output = tree.run(&["check", "--suggestions"]);
        assert_eq!(
            stdout(&output),
            stdout(&expected),
            "[{hosts}]: {}",
            streams(&output)
        );
        assert_eq!(output.status.code(), expected.status.code(), "[{hosts}]");
    }
}
