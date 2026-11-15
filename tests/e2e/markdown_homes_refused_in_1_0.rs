//! The six version-2 load warnings grund 0.23.0 gives, each naming 1.0.0, are refused at load
//! from grund 1.0.0 and their codes are spent (§FS-config-v2.checks.config-md-not-markdown,
//! §FS-config-v2.checks.config-index-not-markdown, §FS-config-v2.checks.config-rule-file-not-markdown,
//! §FS-config-v2.checks.config-rule-folder-not-markdown, §FS-config-v2.checks.config-value-home-absent,
//! §FS-config-v2.checks.config-value-home-invalid), in the order §FS-config-v2.reader.4.order gives;
//! `rule-folder-file-not-markdown` stays a warning
//! (§FS-config-v2.checks.rule-folder-file-not-markdown) and version 1 is unchanged
//! (§FS-config.5.2). Each tree is built in the temp directory: the refusal text is pinned here,
//! outside the release scan, because the golden of a case that printed it before the code does
//! would leave the tree no release to be cut as.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tells apart the trees tests running in parallel build from one case.
static TREES: AtomicUsize = AtomicUsize::new(0);

/// The suffix every one of the six refusals ends with.
const BECAME: &str = "; this became an error in grund 1.0.0";

/// One project tree under the temp directory, removed on drop. The project is `root/project`, so
/// a symlink may lead out of it into `root`.
struct Tree {
    root: PathBuf,
    project: PathBuf,
}

impl Tree {
    /// A project holding `config` as its `grund.toml` and each `(path, text)` of `files`.
    fn new(name: &str, config: &str, files: &[(&str, &str)]) -> Self {
        let n = TREES.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("grund-1397-{name}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let project = root.join("project");
        fs::create_dir_all(&project).expect("create the project");
        let tree = Self { root, project };
        tree.write("grund.toml", config);
        for (path, text) in files {
            tree.write(path, text);
        }
        tree
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.project.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, text).expect("write a file");
    }

    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.project)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            args: args.join(" "),
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Run {
    args: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn shown(&self) -> String {
        format!(
            "grund {}\nexit {}\nstdout:\n{}\nstderr:\n{}",
            self.args, self.code, self.stdout, self.stderr
        )
    }

    fn has_line(&self, line: &str) -> usize {
        self.stderr.lines().filter(|seen| *seen == line).count()
    }
}

/// One of the six, as a project that 0.23.0 loads with its warning: the config, the files it
/// needs, and the refusal line 1.0.0 prints for it.
struct Case {
    name: &'static str,
    code: &'static str,
    config: &'static str,
    files: &'static [(&'static str, &'static str)],
    /// The line of the config the refusal names, the line the warning names.
    line: usize,
    /// The refusal's words: the warning's up to the remedy with its "until then" clause dropped,
    /// then the remedy, with no release.
    words: &'static str,
}

impl Case {
    fn refusal(&self) -> String {
        format!("error: grund.toml:{}: {}{BECAME}", self.line, self.words)
    }

    fn tree(&self) -> Tree {
        Tree::new(self.name, self.config, self.files)
    }
}

const FS_RS: (&str, &str) = ("fs/a.rs", "/// FS-a: A\npub fn a() {}\n");
const FS_MD: (&str, &str) = ("fs/spec.md", "# FS-a: A\n");
const VALUE_JSON: (&str, &str) = ("values/a.json", "{\"CONST-a\": [\"one\"]}\n");

/// Every refusal of the six, the two rows grund 0.22.x already refused included
/// (§FS-config-v2.checks.config-index-not-markdown, §FS-config-v2.checks.config-rule-file-not-markdown).
const CASES: &[Case] = &[
    Case {
        name: "md",
        code: "config-md-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"markdown\", \"rust\"]\n\n[schema.sources.extensions]\n\
                 md = \"rust\"\n\n[schema.kinds.FS]\nfiles = [\"fs/a.rs\"]\n",
        files: &[FS_RS],
        line: 8,
        words: "[schema.sources.extensions] maps `md` to `rust`, which does not read Markdown; \
                `.md` is always Markdown; drop the entry, or map `md` to a `document` language",
    },
    Case {
        name: "index-md-unadmitted",
        code: "config-index-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"rust\"]\n\n[schema.kinds.FS]\nfolders = [\"fs\"]\n\
                 index = \"index.md\"\n",
        files: &[FS_RS, ("fs/index.md", "# Index\n")],
        line: 9,
        words: "`[schema.kinds.FS] index` must name a Markdown file; `index.md` is not read as \
                Markdown; name a file a `document` language reads, or set `index = false`",
    },
    Case {
        name: "index-x-rs",
        code: "config-index-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"markdown\", \"rust\"]\n\n[schema.kinds.FS]\n\
                 folders = [\"fs\"]\nindex = \"x.rs\"\n",
        files: &[("fs/a.md", "# FS-a: A\n"), ("fs/x.rs", "// index\n")],
        line: 9,
        words: "`[schema.kinds.FS] index` must name a Markdown file; `x.rs` is not read as \
                Markdown; name a file a `document` language reads, or set `index = false`",
    },
    Case {
        name: "rule-file-beside",
        code: "config-rule-file-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"markdown\", \"rust\"]\n\n[schema.kinds.FS]\n\
                 files = [\"fs/spec.md\"]\n\n[schema.kinds.RULE]\n\
                 files = [\"rules/rules.md\", \"rules/more.rs\"]\nform = \"rule\"\n",
        files: &[
            FS_MD,
            ("rules/rules.md", "# Rules\n"),
            ("rules/more.rs", "// more\n"),
        ],
        line: 11,
        words: "[schema.kinds.RULE] files entry `rules/more.rs` is read as `rust`, which does not \
                read Markdown, and every home of a rule kind must be; move the rules into a file \
                a `document` language reads",
    },
    Case {
        name: "rule-file-first",
        code: "config-rule-file-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"markdown\", \"rust\"]\n\n[schema.kinds.FS]\n\
                 files = [\"fs/spec.md\"]\n\n[schema.kinds.RULE]\n\
                 files = [\"rules/rules.rs\"]\nform = \"rule\"\n",
        files: &[FS_MD, ("rules/rules.rs", "// rules\n")],
        line: 11,
        words: "[schema.kinds.RULE] files entry `rules/rules.rs` is read as `rust`, which does \
                not read Markdown, and every home of a rule kind must be; move the rules into a \
                file a `document` language reads",
    },
    Case {
        name: "rule-folder-gitkeep",
        code: "config-rule-folder-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"rust\"]\n\n[schema.kinds.FS]\nfiles = [\"fs/a.rs\"]\n\n\
                 [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n",
        files: &[FS_RS, ("rules/.gitkeep", "")],
        line: 11,
        words: "[schema.kinds.RULE] folders entry `rules` can hold no file read as Markdown, \
                because no admitted language reads Markdown; admit one in [schema.sources] \
                languages, such as `markdown`",
    },
    Case {
        name: "rule-folder-absent",
        code: "config-rule-folder-not-markdown",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
                 languages = [\"rust\"]\n\n[schema.kinds.FS]\nfiles = [\"fs/a.rs\"]\n\n\
                 [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n",
        files: &[FS_RS],
        line: 11,
        words: "[schema.kinds.RULE] folders entry `rules` can hold no file read as Markdown, \
                because no admitted language reads Markdown; admit one in [schema.sources] \
                languages, such as `markdown`",
    },
    Case {
        name: "value-absent-files",
        code: "config-value-home-absent",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
                 files = [\"values/a.json\", \"values/absent.json\"]\nform = \"value\"\n",
        files: &[VALUE_JSON],
        // The line that lists the entry, not the `form` line after it.
        line: 5,
        words: "value home for kind `CONST` does not exist: values/absent.json; create the file, \
                or drop the entry",
    },
    Case {
        name: "value-absent-folders",
        code: "config-value-home-absent",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
                 folders = [\"values/one\", \"values/absent\"]\nform = \"value\"\n",
        files: &[("values/one/a.json", "{\"CONST-a\": [\"one\"]}\n")],
        line: 5,
        words: "value home for kind `CONST` does not exist: values/absent; create the folder, or \
                drop the entry",
    },
    Case {
        name: "value-invalid-shape",
        code: "config-value-home-invalid",
        config: "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
                 form = \"value\"\nfiles = [\"values/a.json\", \"values/sub\"]\n",
        files: &[VALUE_JSON, ("values/sub/.gitkeep", "")],
        line: 6,
        words: "value home for kind `CONST` must be an existing file: values/sub; make it an \
                existing file, or drop the entry",
    },
];

/// §FS-config-v2.checks: each of the six is refused at load at its warning's own line, exit `2`
/// on `check` and on `fmt`, which never showed the warning, and `1` on `config validate`, in one
/// line that names 1.0.0 and carries no "until then" clause (§FS-config.4.3).
#[test]
fn each_of_the_six_is_refused_at_load_on_every_command() {
    for case in CASES {
        let tree = case.tree();
        for (args, code) in [
            (&["check"][..], 2),
            (&["config", "validate"], 1),
            (&["fmt", "--check"], 2),
            (&["show", "FS-a"], 2),
        ] {
            let run = tree.run(args);
            assert_eq!(run.code, code, "{}: {}", case.name, run.shown());
            assert_eq!(
                run.has_line(&case.refusal()),
                1,
                "{}: {}",
                case.name,
                run.shown()
            );
            assert!(
                !run.stderr.contains("until then"),
                "{}: {}",
                case.name,
                run.shown()
            );
        }
    }
}

/// §FS-config-v2.checks: on `check` the refusal is the whole of stderr, byte for byte, and a later
/// value home's refusal is that one line, with no `hint:` line beside it
/// (§FS-config-v2.checks.config-value-home-absent).
#[test]
fn each_refusal_is_these_exact_bytes() {
    let mut wrong = Vec::new();
    for case in CASES {
        let run = case.tree().run(&["check"]);
        if run.stderr != format!("{}\n", case.refusal()) || !run.stdout.is_empty() {
            wrong.push(format!(
                "{}: expected\n{}\n{}",
                case.name,
                case.refusal(),
                run.shown()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-config-v2.checks.config-value-home-invalid: a later home that resolves outside the root
/// through a symlink is refused at the line that lists it.
#[test]
#[cfg(unix)]
fn a_later_value_home_outside_the_root_is_refused() {
    let tree = Tree::new(
        "value-invalid-outside",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
         form = \"value\"\nfiles = [\"values/a.json\", \"values/link.json\"]\n",
        &[VALUE_JSON],
    );
    fs::write(tree.root.join("outside.json"), "{\"CONST-b\": [\"two\"]}\n").expect("write");
    std::os::unix::fs::symlink("../../outside.json", tree.project.join("values/link.json"))
        .expect("link out of the root");
    let refusal = format!(
        "error: grund.toml:6: value home for kind `CONST` must normalize inside the project root: \
         values/link.json; name a path inside the project root, or drop the entry{BECAME}"
    );
    for (args, code) in [(&["check"][..], 2), (&["config", "validate"], 1)] {
        let run = tree.run(args);
        assert_eq!(run.code, code, "{}", run.shown());
        assert_eq!(run.stderr, format!("{refusal}\n"), "{}", run.shown());
    }
}

/// §FS-config-v2.reader.4.order: two of the six in one file, and the earliest line is refused, in
/// whichever order the file writes them.
#[test]
fn the_earliest_of_the_six_is_refused() {
    let value_first = Tree::new(
        "order-value-first",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
         files = [\"values/a.json\", \"values/absent.json\"]\nform = \"value\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\n\
         [schema.sources.extensions]\nmd = \"rust\"\n",
        &[VALUE_JSON],
    );
    let run = value_first.run(&["check"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!(
            "error: grund.toml:5: value home for kind `CONST` does not exist: \
             values/absent.json; create the file, or drop the entry{BECAME}\n"
        ),
        "{}",
        run.shown()
    );
    let md_first = Tree::new(
        "order-md-first",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
         languages = [\"markdown\", \"rust\"]\n\n[schema.sources.extensions]\nmd = \"rust\"\n\n\
         [schema.kinds.CONST]\nfiles = [\"values/a.json\", \"values/absent.json\"]\n\
         form = \"value\"\n",
        &[VALUE_JSON],
    );
    let run = md_first.run(&["check"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!("{}\n", CASES[0].refusal()),
        "{}",
        run.shown()
    );
}

/// §FS-config-v2.reader.4.order: an older refusal on any row wins over one of the six on an earlier
/// line, because the six are asked only once every older question has its answer. A guard:
/// green before the change, where the older refusal is the only error.
#[test]
fn an_older_refusal_on_a_later_row_wins() {
    let tree = Tree::new(
        "order-older-later",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.CONST]\n\
         files = [\"values/a.json\", \"values/absent.json\"]\nform = \"value\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\"]\n\n[schema.kinds.RULE]\n\
         files = [\"rules/other.json\"]\nform = \"rule\"\n",
        &[VALUE_JSON, ("rules/other.json", "{}\n")],
    );
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(
        run.stderr,
        "error: grund.toml:13: kind `RULE` sets `rules = true` but rule kinds must be citable, \
         scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`\n",
        "{}",
        run.shown()
    );
}

/// §FS-config-v2.reader.4.order, the owner's guard on #1397: a rule row whose later entry is read as
/// source and whose last is read by no admitted language keeps the rule-kind refusal at `form`,
/// which grund 0.22.x gave it, naming no release. A control: green before the change.
#[test]
fn a_rule_row_refused_before_keeps_its_refusal_at_form() {
    let tree = Tree::new(
        "three-entry",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
         languages = [\"markdown\", \"rust\"]\n\n[schema.kinds.FS]\nfiles = [\"fs/spec.md\"]\n\n\
         [schema.kinds.RULE]\n\
         files = [\"rules/rules.md\", \"rules/more.rs\", \"rules/other.json\"]\n\
         form = \"rule\"\n",
        &[
            FS_MD,
            ("rules/rules.md", "# Rules\n"),
            ("rules/more.rs", "// more\n"),
            ("rules/other.json", "{}\n"),
        ],
    );
    for (args, code) in [(&["check"][..], 2), (&["config", "validate"], 1)] {
        let run = tree.run(args);
        assert_eq!(run.code, code, "{}", run.shown());
        assert_eq!(
            run.stderr,
            "error: grund.toml:12: kind `RULE` sets `rules = true` but rule kinds must be \
             citable, scanned Markdown kinds with a `file` or `folder` home and without `values` \
             or `fetch`\n",
            "{}",
            run.shown()
        );
    }
}

/// §FS-config-v2.checks.config-md-not-markdown: a workspace member's refusal names the member's
/// own file.
#[test]
fn a_workspace_member_refusal_names_the_members_file() {
    let case = &CASES[0];
    let tree = Tree::new(
        "member",
        "grund_config_version = 2\nproject_name = \"workspace\"\n\n[workspace]\n\
         members = [\"mem\"]\n",
        &[("mem/grund.toml", case.config), ("mem/fs/a.rs", FS_RS.1)],
    );
    let refusal = case.refusal().replacen("grund.toml", "mem/grund.toml", 1);
    for (args, code) in [(&["check"][..], 2), (&["config", "validate"], 1)] {
        let run = tree.run(args);
        assert_eq!(run.code, code, "{}", run.shown());
        assert_eq!(run.has_line(&refusal), 1, "{}", run.shown());
    }
}

/// A clean version-2 project that loads, for asking the selectors about the spent codes.
fn clean_tree(name: &str) -> Tree {
    Tree::new(
        name,
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.kinds.FS]\n\
         files = [\"fs/spec.md\"]\n",
        &[("fs/spec.md", "# FS-a: A\n\nSee \u{a7}FS-a.\n")],
    )
}

/// The six codes, each spent from 1.0.0.
const CODES: [&str; 6] = [
    "config-md-not-markdown",
    "config-index-not-markdown",
    "config-rule-file-not-markdown",
    "config-rule-folder-not-markdown",
    "config-value-home-absent",
    "config-value-home-invalid",
];

/// §FS-config-v2.checks.config-md-not-markdown: each code is spent and kept, so `--only` and
/// `--ignore` are still accepted and select nothing, and `--only` says the check is spent
/// (§FS-check.1.4.1).
#[test]
fn each_code_is_spent_and_its_selectors_still_accepted() {
    let tree = clean_tree("spent");
    for code in CODES {
        let run = tree.run(&["check", "--only", code]);
        assert_eq!(run.code, 0, "{}", run.shown());
        assert_eq!(
            run.stderr,
            format!(
                "warning: --only {code} judged nothing \u{2014} the check is spent from grund \
                 1.0.0 and never fires\n"
            ),
            "{}",
            run.shown()
        );
        let run = tree.run(&["check", "--ignore", code]);
        assert_eq!(run.code, 0, "{}", run.shown());
        assert_eq!(run.stderr, "", "{}", run.shown());
        assert_eq!(run.stdout, "success\n", "{}", run.shown());
    }
}

/// §FS-config-v2.checks.config-md-not-markdown: no selector suppresses a load refusal.
#[test]
fn no_selector_suppresses_the_refusal() {
    for case in CASES {
        let tree = case.tree();
        for flag in ["--only", "--ignore"] {
            let run = tree.run(&["check", flag, case.code]);
            assert_eq!(run.code, 2, "{}: {}", case.name, run.shown());
            assert_eq!(
                run.has_line(&case.refusal()),
                1,
                "{}: {}",
                case.name,
                run.shown()
            );
        }
    }
}

/// §FS-config-v2.checks.config-md-not-markdown: `config explain` answers for each code as spent,
/// on any tree, one whose config is refused included.
#[test]
fn explain_answers_for_each_code_as_spent() {
    let tree = CASES[0].tree();
    for code in CODES {
        let run = tree.run(&["config", "explain", code]);
        assert_eq!(run.code, 0, "{}", run.shown());
        for line in [
            "Severity: none \u{2014} spent from grund 1.0.0 and never fires.",
            "Fix: none \u{2014} it never fires",
        ] {
            assert!(
                run.stdout.lines().any(|seen| seen == line),
                "{code}: {line}\n{}",
                run.shown()
            );
        }
        assert!(!run.stdout.contains("Ramp:"), "{code}\n{}", run.shown());
    }
}

/// §FS-config-v2.checks.rule-folder-file-not-markdown: a file a `source` language reads in a rule
/// folder, with `md` left to Markdown, stays a located warning on stdout and exit `0`, naming no
/// release. A control: green before the change.
#[test]
fn control_a_source_file_in_a_rule_folder_stays_a_warning() {
    let tree = Tree::new(
        "rule-folder-file",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
         languages = [\"markdown\", \"rust\"]\n\n[schema.kinds.FS]\nfiles = [\"fs/spec.md\"]\n\n\
         [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n",
        &[
            ("fs/spec.md", "# FS-a: A\n\nSee \u{a7}FS-a.\n"),
            ("rules/r.rs", "// a rule file read as rust\n"),
        ],
    );
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    assert!(
        run.stdout.lines().any(|line| line.starts_with(
            "rules/r.rs:1: warning: read as `rust`, which does not read Markdown, in `rules`"
        )),
        "{}",
        run.shown()
    );
    assert!(!run.stdout.contains("1.0.0"), "{}", run.shown());
    assert!(!run.stderr.contains("1.0.0"), "{}", run.shown());
    let run = tree.run(&["config", "validate"]);
    assert_eq!(run.code, 0, "{}", run.shown());
}

/// §FS-config.5.2: a version 1 rule folder in a project that reads no Markdown loads as before.
/// A control: green before the change.
#[test]
fn control_version_1_is_unchanged() {
    let tree = Tree::new(
        "v1",
        "grund_config_version = 1\nproject_name = \"r1397\"\n\n[scan]\nextensions = [\"rs\"]\n\n\
         [[kinds]]\nkind = \"FS\"\nfile = \"fs/a.rs\"\n\n[[kinds]]\nkind = \"RULE\"\n\
         folder = \"rules\"\nrules = true\n",
        &[
            ("fs/a.rs", "/// FS-1-a: A\npub fn a() {}\n"),
            ("rules/.gitkeep", ""),
        ],
    );
    for (args, code) in [(&["check"][..], 0), (&["config", "validate"], 0)] {
        let run = tree.run(args);
        assert_eq!(run.code, code, "{}", run.shown());
        assert!(!run.stderr.contains("1.0.0"), "{}", run.shown());
    }
}

/// §FS-config-v2.checks.config-index-not-markdown: a row that names no `index` named nothing, so a
/// default `README.md` not read as Markdown is judged as under `index = false`, silently, and
/// `md` mapped to `markdown` or a `document` definition is silent. A control: green before the change.
#[test]
fn control_what_1_0_0_leaves_loading_stays_silent() {
    let default_index = Tree::new(
        "default-index",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
         languages = [\"rust\"]\n\n[schema.kinds.FS]\nfolders = [\"fs\"]\n",
        &[FS_RS, ("fs/README.md", "# FS\n")],
    );
    let document = Tree::new(
        "md-document",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources]\n\
         languages = [\"notes\"]\n\n[schema.sources.extensions]\nmd = \"notes\"\n\n\
         [schema.sources.definitions.notes]\nclass = \"document\"\n\
         comment_prefixes = [\"//\"]\ndoc_comments = \"none\"\ndocstrings = \"off\"\n\n\
         [schema.kinds.FS]\nfiles = [\"fs/spec.md\"]\n",
        &[("fs/spec.md", "# FS-a: A\n\nSee \u{a7}FS-a.\n")],
    );
    let markdown = Tree::new(
        "md-markdown",
        "grund_config_version = 2\nproject_name = \"r1397\"\n\n[schema.sources.extensions]\n\
         md = \"markdown\"\n\n[schema.kinds.FS]\nfiles = [\"fs/spec.md\"]\n",
        &[("fs/spec.md", "# FS-a: A\n\nSee \u{a7}FS-a.\n")],
    );
    for tree in [default_index, document, markdown] {
        for args in [&["check"][..], &["config", "validate"]] {
            let run = tree.run(args);
            assert_ne!(run.code, 2, "{}", run.shown());
            assert!(!run.stderr.contains("1.0.0"), "{}", run.shown());
            assert!(!run.stderr.contains("error:"), "{}", run.shown());
        }
    }
}
