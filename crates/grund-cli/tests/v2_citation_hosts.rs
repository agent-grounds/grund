//! §REQ-no-missed-citation.3 under v2: every comment host of
//! §FS-config-v2.schema.sources.epoch, each written in every form its prefix stands
//! for (§FS-languages.prefixes.forms), carries a dangling marked citation, and
//! `grund check` must report each one at its line, and nothing else. A dangling
//! citation is what proves the line was read: a missed citation also passes.
//! Every epoch suffix appears at least once, the opt-in rows are named, and a
//! custom definition reads through its own prefix
//! (§FS-config-v2.schema.sources.definitions). Each form stands alone behind a
//! blank line, so none is read as a neighbour's continuation. The trees keep the
//! default `doc-comments`, under which most of these comments declare nothing,
//! yet each citation is still checked (§FS-config-v2.schema.sources.declarations).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every source row of the epoch, the opt-in rows named, and one custom
/// language read through `%%`, so each test's files take their own rows.
const CONFIG: &str = concat!(
    "grund_config_version = 2\n",
    "project_name = \"fixture\"\n",
    "[schema.sources]\n",
    "languages = [\"rust\", \"c\", \"cpp\", \"java\", \"kotlin\", \"scala\", \"swift\",\n",
    "  \"javascript\", \"typescript\", \"csharp\", \"php\", \"python\", \"go\", \"ruby\",\n",
    "  \"sql\", \"lua\", \"haskell\", \"lisp\", \"scheme\", \"clojure\", \"ada\",\n",
    "  \"shell\", \"r\", \"objectivec\", \"dart\", \"notes\"]\n",
    "[schema.sources.extensions]\n",
    "note = \"notes\"\n",
    "[schema.sources.definitions.notes]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"%%\"]\n",
    "[schema.kinds.FS]\n",
);

/// One fixture line: the text before the citation, the name of the host form
/// it is written in (empty for a line that cites nothing), and the text after.
type Line = (&'static str, &'static str, &'static str);

/// A blank line, which ends a comment block: each form below stands alone, so
/// it opens its own comment rather than continuing a neighbour's.
const GAP: Line = ("", "", "");

/// `//` and the forms it stands for.
const SLASH: &[Line] = &[
    ("// See ", "line", "."),
    GAP,
    ("/// See ", "line-doc", "."),
    GAP,
    ("//! See ", "line-inner", "."),
    GAP,
];

/// `/*` with the forms it stands for, and `*` continuing a block.
const BLOCK: &[Line] = &[
    ("/* See ", "block", ". */"),
    GAP,
    ("/** See ", "block-doc", ". */"),
    GAP,
    ("/*! See ", "block-inner", ". */"),
    GAP,
    ("/**", "", ""),
    (" * See ", "star", "."),
    (" */", "", ""),
    GAP,
];

/// `#`, the host of Python, Ruby, PHP, shell and R.
const HASH: &[Line] = &[("# See ", "hash", "."), GAP];

/// R's roxygen `#'`.
const ROXYGEN: &[Line] = &[("#' See ", "roxygen", "."), GAP];

/// `--` and the forms it stands for: LDoc's `---` and Haddock's markers.
const DASH: &[Line] = &[
    ("-- See ", "dash", "."),
    GAP,
    ("--- See ", "dash-ldoc", "."),
    GAP,
    ("-- | See ", "dash-pipe", "."),
    GAP,
    ("--| See ", "dash-pipe-glued", "."),
    GAP,
    ("-- ^ See ", "dash-caret", "."),
    GAP,
    ("--^ See ", "dash-caret-glued", "."),
    GAP,
];

/// `;` stands for any run of semicolons.
const SEMICOLON: &[Line] = &[
    ("; See ", "semi", "."),
    GAP,
    (";; See ", "semi-two", "."),
    GAP,
    (";;; See ", "semi-three", "."),
    GAP,
    (";;;; See ", "semi-four", "."),
    GAP,
];

/// Python's `#`, and a module docstring, a function's, and a multi-line
/// docstring's body.
const PYTHON: &[Line] = &[
    ("\"\"\"See ", "module-docstring", ".\"\"\""),
    GAP,
    ("# See ", "hash", "."),
    ("def f():", "", ""),
    ("    \"\"\"See ", "function-docstring", ".\"\"\""),
    ("    return 1", "", ""),
    GAP,
    ("def g():", "", ""),
    ("    \"\"\"Summary.", "", ""),
    GAP,
    ("    See ", "docstring-body", "."),
    ("    \"\"\"", "", ""),
    ("    return 2", "", ""),
];

/// The custom language's literal prefix.
const PERCENT: &[Line] = &[("%% See ", "percent", ".")];

/// A PHP file's opening tag, a Go package clause, a Ruby method: a code line.
const PHP_OPEN: &[Line] = &[("<?php", "", "")];
const GO_PACKAGE: &[Line] = &[("package m", "", "")];
const RUBY_DEF: &[Line] = &[("def a; end", "", "")];

/// A file under `src/<stem>.<suffix>` and the error each of its cited lines must
/// draw: the citation of form `f` is `FS-missing-<stem>-<f>`, unique per file.
fn plant(stem: &str, suffix: &str, lines: &[Line]) -> (String, String, Vec<String>) {
    let path = format!("src/{stem}.{suffix}");
    let mut text = String::new();
    let mut expected = Vec::new();
    for (number, (before, form, after)) in lines.iter().enumerate() {
        if form.is_empty() {
            text.push_str(before);
        } else {
            let id = format!("FS-missing-{stem}-{form}");
            text.push_str(&format!("{before}\u{a7}{id}{after}"));
            expected.push(format!(
                "{path}:{}: error: unknown reference {id}",
                number + 1
            ));
        }
        text.push('\n');
    }
    (path, text, expected)
}

/// Writes `files` beside `CONFIG` in a fresh tree, runs `grund check .` there,
/// and requires its error lines to be exactly the planted citations' errors.
fn dangling(name: &str, files: &[(&str, &str, Vec<Line>)]) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("v2_citation_hosts")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    write(&root, "grund.toml", CONFIG);
    let mut expected = Vec::new();
    for (stem, suffix, lines) in files {
        let (path, text, errors) = plant(stem, suffix, lines);
        write(&root, &path, &text);
        expected.extend(errors);
    }
    expected.sort();
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["check", "."])
        .current_dir(&root)
        .output()
        .expect("spawn grund");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut got: Vec<String> = stdout
        .lines()
        .filter(|line| line.contains(": error: "))
        .map(str::to_string)
        .collect();
    got.sort();
    assert_eq!(
        got, expected,
        "{name}: every planted citation is reported, and nothing else:\n{stdout}{stderr}"
    );
    assert_eq!(output.status.code(), Some(1), "{name}: the check fails");
}

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
    fs::write(&path, text).expect("write fixture file");
}

/// The `//`, `/*` and `*` hosts with every form, over each default c-family
/// suffix; PHP adds its `#`.
#[test]
fn c_family_hosts_report_every_form() {
    let c_family = [SLASH, BLOCK].concat();
    let mut files: Vec<(&str, &str, Vec<Line>)> = [
        ("rust", "rs"),
        ("c", "c"),
        ("cpp", "cpp"),
        ("java", "java"),
        ("kotlin", "kt"),
        ("scala", "scala"),
        ("swift", "swift"),
        ("javascript", "js"),
        ("typescript", "ts"),
        ("tsx", "tsx"),
        ("csharp", "cs"),
    ]
    .into_iter()
    .map(|(stem, suffix)| (stem, suffix, c_family.clone()))
    .collect();
    files.push(("php", "php", [PHP_OPEN, SLASH, BLOCK, HASH].concat()));
    dangling("c-family", &files);
}

/// Go's `//` with the forms it stands for; Go reads no `/*`.
#[test]
fn go_line_hosts_report_every_form() {
    dangling("go", &[("go", "go", [SLASH, GO_PACKAGE].concat())]);
}

/// Python's `#` and its docstrings, module and function, one line or several.
#[test]
fn python_hash_and_docstrings_report() {
    dangling("python", &[("python", "py", PYTHON.to_vec())]);
}

/// Ruby's `#`.
#[test]
fn ruby_hash_reports() {
    let ruby = [HASH, RUBY_DEF].concat();
    dangling("ruby", &[("ruby", "rb", ruby)]);
}

/// `--` with every form, in SQL, Lua, Haskell (literate too) and Ada.
#[test]
fn dash_hosts_report_every_form() {
    let files: Vec<(&str, &str, Vec<Line>)> = [
        ("sql", "sql"),
        ("lua", "lua"),
        ("haskell", "hs"),
        ("literate", "lhs"),
        ("ada", "ada"),
        ("adabody", "adb"),
        ("adaspec", "ads"),
    ]
    .into_iter()
    .map(|(stem, suffix)| (stem, suffix, DASH.to_vec()))
    .collect();
    dangling("dash", &files);
}

/// `;` in every run length, in Lisp, Scheme and Clojure.
#[test]
fn semicolon_hosts_report_every_run() {
    let files: Vec<(&str, &str, Vec<Line>)> =
        [("lisp", "lisp"), ("scheme", "scm"), ("clojure", "clj")]
            .into_iter()
            .map(|(stem, suffix)| (stem, suffix, SEMICOLON.to_vec()))
            .collect();
    dangling("semicolon", &files);
}

/// The opt-in rows once named: shell's `#`, R's `#` and roxygen `#'` under
/// both suffixes, and the c-family hosts of Objective-C and Dart.
#[test]
fn opt_in_rows_report_once_named() {
    let c_family = [SLASH, BLOCK].concat();
    let r = [HASH, ROXYGEN].concat();
    dangling(
        "opt-in",
        &[
            ("sh", "sh", HASH.to_vec()),
            ("bash", "bash", HASH.to_vec()),
            ("zsh", "zsh", HASH.to_vec()),
            ("rlower", "r", r.clone()),
            ("rupper", "R", r),
            ("objc", "m", c_family.clone()),
            ("objcpp", "mm", c_family.clone()),
            ("dart", "dart", c_family),
        ],
    );
}

/// A definition's `%%` reads its citations.
#[test]
fn a_custom_prefix_reports() {
    dangling("custom", &[("notes", "note", PERCENT.to_vec())]);
}
