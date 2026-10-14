//! §FS-config-v2.schema.sources.declarations — which comments declare an ID, one family of
//! §FS-inline-citation-style.1.1.4 at a time. Each tree plants declaration-shaped
//! lines in comments the family's doc profile admits and in comments it does not,
//! with no citation anywhere, so eligibility cannot lean on the inline-style pass
//! (§FS-config-v2.schema.sources.declarations.gate). Under v2's default `doc-comments` only the
//! admitted ones are listed; the v1 twin of each tree keeps every comment's
//! declaration (§FS-config.5.2), which is what `any-comment` means in v2.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh tree under Cargo's scratch directory for integration tests.
fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("v2_declaration_hosts")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    for (rel, text) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        fs::write(&path, text).expect("write fixture file");
    }
    root
}

fn grund(args: &[&str], root: &Path) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .arg(root)
        .output()
        .expect("spawn grund");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The `(id, line)` of every declaration `grund list --format json` reports.
fn declared(root: &Path) -> Vec<(String, u64)> {
    let (code, stdout, stderr) = grund(&["list", "--format", "json"], root);
    assert_eq!(code, Some(0), "list failed:\n{stdout}{stderr}");
    stdout
        .lines()
        .map(|line| {
            let row: serde_json::Value = serde_json::from_str(line).expect("one JSON row");
            (
                row["id"].as_str().expect("id").to_string(),
                row["line"].as_u64().expect("line"),
            )
        })
        .collect()
}

/// A v2 config admitting `languages`, with `extra` written under `[schema.sources]`
/// and one place-less `FS` row, so the scan is the config root.
fn v2(languages: &str, extra: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"fixture\"\n[schema.sources]\n\
         languages = [{languages}]\n{extra}[schema.kinds.FS]\n"
    )
}

/// The v1 twin: one home-less `FS` kind, every file under `src/` scanned with
/// v1's default extensions and prefixes unless `scan` adds keys.
fn v1(scan: &str) -> String {
    format!(
        "grund_config_version = 1\nproject_name = \"fixture\"\n[reference]\nstrict = true\n\
         [id]\nformat = \"{{kind}}-{{slug}}\"\n[[kinds]]\nkind = \"FS\"\n[scan]\n\
         include = [\"src\"]\n{scan}"
    )
}

fn ids(expected: &[(&str, u64)]) -> Vec<(String, u64)> {
    let mut ids: Vec<(String, u64)> = expected
        .iter()
        .map(|(id, line)| (id.to_string(), *line))
        .collect();
    ids.sort();
    ids
}

/// Runs one family: `file` at `src/<path>` under v1 and under v2, and requires
/// each to declare exactly the listed IDs. v1 runs first: its half holds today.
fn family(
    name: &str,
    v2_config: &str,
    v1_config: &str,
    path: &str,
    file: &str,
    under_v2: &[(&str, u64)],
    under_v1: &[(&str, u64)],
) {
    let rel = format!("src/{path}");
    let v1_root = tree(
        &format!("{name}-v1"),
        &[("grund.toml", v1_config), (&rel, file)],
    );
    let mut got = declared(&v1_root);
    got.sort();
    assert_eq!(
        got,
        ids(under_v1),
        "{name}: v1 keeps every comment's declaration"
    );
    let v2_root = tree(
        &format!("{name}-v2"),
        &[("grund.toml", v2_config), (&rel, file)],
    );
    let mut got = declared(&v2_root);
    got.sort();
    assert_eq!(got, ids(under_v2), "{name}: v2 doc-comments declarations");
}

const RUST: &str = concat!(
    "//! FS-module: Module doc\n",
    "\n",
    "/// FS-line-doc: Line doc\n",
    "pub fn a() {\n",
    "    /// FS-dangling: A dangling doc marker qualifies\n",
    "    let x = 1; // FS-trailing: A trailing comment\n",
    "    // FS-plain: A plain comment\n",
    "}\n",
    "/** FS-block-doc: Block doc */\n",
    "pub fn b() {}\n",
    "/*! FS-inner-block: Inner block doc */\n",
    "/* FS-plain-block: A plain block */\n",
    "pub fn c() {}\n",
    "//// FS-rule-line: A rule line is no doc marker\n",
    "pub fn d() {}\n",
);

/// §FS-inline-citation-style.1.1.4 `c-family`: `//!`, `///` dangling or not,
/// `/**` and `/*!` declare; plain, trailing, block and `////` comments do not.
#[test]
fn c_family_markers_declare_and_plain_comments_do_not() {
    family(
        "rust",
        &v2("\"rust\"", ""),
        &v1(""),
        "lib.rs",
        RUST,
        &[
            ("FS-module", 1),
            ("FS-line-doc", 3),
            ("FS-dangling", 5),
            ("FS-block-doc", 9),
            ("FS-inner-block", 11),
        ],
        &[
            ("FS-module", 1),
            ("FS-line-doc", 3),
            ("FS-dangling", 5),
            ("FS-plain", 7),
            ("FS-block-doc", 9),
            ("FS-inner-block", 11),
            ("FS-plain-block", 12),
        ],
    );
}

/// §FS-config-v2.schema.sources.extensions: a remapped suffix takes the target's
/// doc profile with its reader — `.h` read as C declares in `///` only.
#[test]
fn a_remapped_suffix_takes_the_target_profile() {
    family(
        "remap",
        &v2("\"c\"", "[schema.sources.extensions]\nh = \"c\"\n"),
        &v1(""),
        "m.h",
        "/// FS-h-doc: Doc\nint a(void);\n// FS-h-plain: Plain\nint b(void);\n",
        &[("FS-h-doc", 1)],
        &[("FS-h-doc", 1), ("FS-h-plain", 3)],
    );
}

const PYTHON: &str = concat!(
    "\"\"\"FS-py-module: Module docstring\"\"\"\n",
    "# FS-py-hash: A hash comment above a def\n",
    "def f():\n",
    "    \"\"\"FS-py-docstring: Function docstring\"\"\"\n",
    "    # FS-py-inner: An inner hash comment\n",
    "    return 1\n",
    "DATA = \"\"\"FS-py-data: Assigned data\"\"\"\n",
);

/// §FS-inline-citation-style.1.1.4 `python`: docstrings declare, lexically and
/// without assigned data; `#` blocks do not.
#[test]
fn python_docstrings_declare_and_hash_comments_do_not() {
    family(
        "python",
        &v2("\"python\"", ""),
        &v1(""),
        "m.py",
        PYTHON,
        &[("FS-py-module", 1), ("FS-py-docstring", 4)],
        &[
            ("FS-py-module", 1),
            ("FS-py-hash", 2),
            ("FS-py-docstring", 4),
            ("FS-py-inner", 5),
        ],
    );
}

const GO: &str = concat!(
    "// FS-go-file: The leading file block\n",
    "package m\n",
    "\n",
    "// FS-go-detached: A detached block\n",
    "\n",
    "func a() {\n",
    "\t// FS-go-local-var: A block above a local var\n",
    "\tvar x = 1\n",
    "\t// FS-go-inner: An inner note\n",
    "\tx++\n",
    "}\n",
    "// FS-go-func: Adjacent to func\n",
    "func b() {}\n",
    "// FS-go-type: Adjacent to type\n",
    "type T int\n",
);

/// §FS-inline-citation-style.1.1.4 `go`: the leading block and blocks adjacent to
/// a starter declare, a local `var` included; detached and inner blocks do not.
#[test]
fn go_adjacency_and_the_leading_block_declare() {
    family(
        "go",
        &v2("\"go\"", ""),
        &v1(""),
        "m.go",
        GO,
        &[
            ("FS-go-file", 1),
            ("FS-go-local-var", 7),
            ("FS-go-func", 12),
            ("FS-go-type", 14),
        ],
        &[
            ("FS-go-file", 1),
            ("FS-go-detached", 4),
            ("FS-go-local-var", 7),
            ("FS-go-inner", 9),
            ("FS-go-func", 12),
            ("FS-go-type", 14),
        ],
    );
}

const RUBY: &str = concat!(
    "# FS-rb-file: The leading file block\n",
    "\n",
    "class A\n",
    "  # FS-rb-def: Adjacent to def\n",
    "  def a; end\n",
    "  # FS-rb-private: A private def does not qualify\n",
    "  private def b; end\n",
    "  # FS-rb-inner: An inner note\n",
    "  x = 1\n",
    "end\n",
);

/// §FS-inline-citation-style.1.1.4 `ruby`: `class`, `module` and `def` adjacency
/// and the leading block declare; `private def` and inner notes do not.
#[test]
fn ruby_adjacency_declares_and_private_def_does_not() {
    family(
        "ruby",
        &v2("\"ruby\"", ""),
        &v1(""),
        "m.rb",
        RUBY,
        &[("FS-rb-file", 1), ("FS-rb-def", 4)],
        &[
            ("FS-rb-file", 1),
            ("FS-rb-def", 4),
            ("FS-rb-private", 6),
            ("FS-rb-inner", 8),
        ],
    );
}

const SQL: &str = concat!(
    "SELECT 0;\n",
    "\n",
    "-- FS-sql-create: Adjacent to create\n",
    "CREATE OR REPLACE FUNCTION f() RETURNS int AS 'select 1' LANGUAGE sql;\n",
    "\n",
    "-- FS-sql-select: Above a select\n",
    "SELECT 1;\n",
);

/// §FS-inline-citation-style.1.1.4 `sql`: a block above `create` declares.
#[test]
fn sql_create_adjacency_declares() {
    family(
        "sql",
        &v2("\"sql\"", ""),
        &v1(""),
        "m.sql",
        SQL,
        &[("FS-sql-create", 3)],
        &[("FS-sql-create", 3), ("FS-sql-select", 6)],
    );
}

const LUA: &str = concat!(
    "local x = 1\n",
    "--- FS-lua-ldoc: An LDoc block\n",
    "local function f() end\n",
    "-- FS-lua-plain: A plain block\n",
    "local y = 2\n",
);

/// §FS-inline-citation-style.1.1.4 `ldoc`: a `---` block declares, `--` does not.
#[test]
fn lua_ldoc_declares_and_plain_comments_do_not() {
    family(
        "lua",
        &v2("\"lua\"", ""),
        &v1(""),
        "m.lua",
        LUA,
        &[("FS-lua-ldoc", 2)],
        &[("FS-lua-ldoc", 2), ("FS-lua-plain", 4)],
    );
}

const HASKELL: &str = concat!(
    "module M where\n",
    "-- | FS-hs-haddock: A Haddock block\n",
    "f = 1\n",
    "-- FS-hs-plain: A plain block\n",
    "g = 2\n",
    "-- ^ FS-hs-caret: A caret Haddock block\n",
    "h = 3\n",
);

/// §FS-inline-citation-style.1.1.4 `haddock`: `-- |` and `-- ^` declare.
#[test]
fn haskell_haddock_declares_and_plain_comments_do_not() {
    family(
        "haskell",
        &v2("\"haskell\"", ""),
        &v1(""),
        "M.hs",
        HASKELL,
        &[("FS-hs-haddock", 2), ("FS-hs-caret", 6)],
        &[("FS-hs-haddock", 2), ("FS-hs-plain", 4), ("FS-hs-caret", 6)],
    );
}

/// §FS-config-v2.schema.sources.declarations.eligible: a language whose profile is `none` — Lisp
/// and Ada here — declares in any configured comment, so it can still declare.
#[test]
fn a_language_with_no_doc_notation_still_declares() {
    family(
        "lisp",
        &v2("\"lisp\"", ""),
        &v1(""),
        "m.lisp",
        "(let ((x 1)) x)\n;; FS-lisp-any: Any comment declares\n(print 1)\n",
        &[("FS-lisp-any", 2)],
        &[("FS-lisp-any", 2)],
    );
    family(
        "ada",
        &v2("\"ada\"", ""),
        &v1(""),
        "m.adb",
        "procedure P is\nbegin\n   -- FS-ada-any: Any comment declares\n   null;\nend P;\n",
        &[("FS-ada-any", 3)],
        &[("FS-ada-any", 3)],
    );
}

/// §FS-config-v2.schema.sources.definitions: a prefix-only definition has the
/// profile `none`, so each of its comments declares; one that names a profile
/// declares only where that profile admits.
#[test]
fn a_custom_definition_declares_by_its_profile() {
    let note = "%% FS-custom-any: A prefix-only language declares\nbody\n";
    family(
        "custom",
        &v2(
            "\"notes\"",
            "[schema.sources.extensions]\nnote = \"notes\"\n\
             [schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"%%\"]\n",
        ),
        &v1("extensions = [\"note\"]\ncomment_prefixes = [\"%%\"]\n"),
        "a.note",
        note,
        &[("FS-custom-any", 1)],
        &[("FS-custom-any", 1)],
    );
    family(
        "custom-profile",
        &v2(
            "\"script\"",
            "[schema.sources.extensions]\nsx = \"script\"\n\
             [schema.sources.definitions.script]\nclass = \"source\"\ncomment_prefixes = [\"//\"]\n\
             doc_comments = \"c-family\"\n",
        ),
        &v1("extensions = [\"sx\"]\ncomment_prefixes = [\"//\"]\n"),
        "a.sx",
        "/// FS-sx-doc: Doc\nrun()\n// FS-sx-plain: Plain\nrun()\n",
        &[("FS-sx-doc", 1)],
        &[("FS-sx-doc", 1), ("FS-sx-plain", 3)],
    );
}

/// §FS-config-v2.schema.sources.epoch: the opt-in rows — shell's function
/// adjacency and R's roxygen `#'` — once named.
#[test]
fn opt_in_shell_and_r_declare_by_their_profiles() {
    family(
        "shell",
        &v2("\"shell\"", ""),
        &v1(""),
        "m.sh",
        "echo hi\n# FS-sh-func: Adjacent to a function\nf() { :; }\n# FS-sh-plain: A plain note\necho x\n",
        &[("FS-sh-func", 2)],
        &[("FS-sh-func", 2), ("FS-sh-plain", 4)],
    );
    family(
        "r",
        &v2("\"r\"", ""),
        &v1(""),
        "m.R",
        "x <- 0\n#' FS-r-rox: A roxygen block\nf <- function() 1\n# FS-r-plain: A plain note\ny <- 1\n",
        &[("FS-r-rox", 2)],
        &[("FS-r-plain", 4)],
    );
}

/// §FS-config-v2.schema.sources.declarations.gate: the near-miss check runs behind the gate —
/// an off-grammar declaration in a plain comment is prose under v2, while the
/// same line in a doc comment is still a near miss.
#[test]
fn near_miss_runs_behind_the_gate() {
    let file =
        "/// FS-Doc_Bad: A doc near miss\npub fn a() {\n    // FS-Note_Bad: A plain near miss\n}\n";
    let root = tree(
        "near-miss-v1",
        &[("grund.toml", &v1("")), ("src/lib.rs", file)],
    );
    let (_, stdout, _) = grund(&["check"], &root);
    assert!(
        stdout.contains("src/lib.rs:1:"),
        "v1: the doc near miss:\n{stdout}"
    );
    assert!(
        stdout.contains("src/lib.rs:3:"),
        "v1: the plain near miss:\n{stdout}"
    );
    let root = tree(
        "near-miss-v2",
        &[("grund.toml", &v2("\"rust\"", "")), ("src/lib.rs", file)],
    );
    let (code, stdout, stderr) = grund(&["check"], &root);
    assert_eq!(
        code,
        Some(1),
        "v2: the doc near miss fails the check:\n{stdout}{stderr}"
    );
    assert!(
        stdout.contains("src/lib.rs:1:"),
        "v2: the doc near miss:\n{stdout}"
    );
    assert!(
        !stdout.contains("src/lib.rs:3:"),
        "v2: a plain comment is no near miss:\n{stdout}"
    );
}
