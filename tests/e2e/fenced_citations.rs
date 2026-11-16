//! A Markdown fence whose tag `[schema.sources] fenced` lists is read for citations as a `<pre>`
//! block is, and stays a fence for everything else (§FS-config-v2.schema.sources.fenced,
//! §FS-languages.fence.language, §FS-citations.recognized.contexts). Its citations resolve, count
//! as uses and are reported where they dangle; no line of it declares, opens a section or toggles
//! `fmt`, and `fmt` rewrites none of it (§FS-fmt.2.3.3, §FS-fmt.2.5.2.2,
//! §FS-citations.rewrite-exemption). The index looks at its lines (§FS-check.3.18.5.1), `config
//! show` prints the key (§FS-config-v2.schema.sources.effective), and the managed block names
//! only the fences left unread (§FS-init.2.3.4.2, §FS-init.2.3.7.1). The verdicts are
//! §DF-fenced-citations; issue #1364 is the report.
//!
//! Every expected output below is what the same text prints today written in a `<pre>` block, or,
//! for `fmt` and `init`, what the tree prints with no `fenced` key, so a red test here fails on
//! the behaviour and not only on the key being refused. These are not goldens because a golden
//! has no skip. The trees are built in the temp directory, so run them with a `TMPDIR` that no git
//! tree or grund config encloses.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The issue's config, with `mermaid` selected.
const SELECTED: &str = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                        fenced = [\"mermaid\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";

/// The same config with no `fenced` key: every fence skipped, as today.
const PLAIN: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\
                     [schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";

/// `docs/ar.md`, the one declaration every tree cites.
const AR: &str = "# AR-gateway: Gateway\n";

/// The issue's `README.md`: a `mermaid` fence citing one real and one dangling ID, then `<pre>`.
const ISSUE: &str = "**Demo**\n\n```mermaid\nflowchart LR\n  \
                     gateway[\"**gateway** \u{a7}AR-gateway\"] --> db[\"**db** \u{a7}AR-nonexistent\"]\n\
                     ```\n\n<pre>\ngateway  \u{a7}AR-nonexistent2\n</pre>\n";

/// [`ISSUE`] with its fence written as a `<pre>` block, line for line and column for column.
const ISSUE_PRE: &str = "**Demo**\n\n<pre>\nflowchart LR\n  \
                         gateway[\"**gateway** \u{a7}AR-gateway\"] --> db[\"**db** \u{a7}AR-nonexistent\"]\n\
                         </pre>\n\n<pre>\ngateway  \u{a7}AR-nonexistent2\n</pre>\n";

/// What `check` prints for [`ISSUE`] once the fence is read, as it prints for [`ISSUE_PRE`] today.
const ISSUE_CHECK: &str = "README.md:5: error: unknown reference AR-nonexistent\n\
                           README.md:9: error: unknown reference AR-nonexistent2\n";

/// `grund cover README.md --format json` for the same: each citation at its line and column.
const ISSUE_COVER: &str = "{\"path\":\"README.md\",\"citations\":[\
{\"path\":\"README.md\",\"line\":5,\"column\":24,\"id\":\"AR-gateway\",\"section\":null,\"marker\":true,\
\"text\":\"\u{a7}AR-gateway\",\"enclosing_declaration\":null,\"enclosing_section\":null},\
{\"path\":\"README.md\",\"line\":5,\"column\":54,\"id\":\"AR-nonexistent\",\"section\":null,\"marker\":true,\
\"text\":\"\u{a7}AR-nonexistent\",\"enclosing_declaration\":null,\"enclosing_section\":null},\
{\"path\":\"README.md\",\"line\":9,\"column\":10,\"id\":\"AR-nonexistent2\",\"section\":null,\"marker\":true,\
\"text\":\"\u{a7}AR-nonexistent2\",\"enclosing_declaration\":null,\"enclosing_section\":null}]}\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// `grund.toml` holding `config`, `docs/ar.md` holding [`AR`], and each `(path, text)`.
    fn new(name: &str, config: &str, files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1364-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        let tree = Self(root);
        tree.write("grund.toml", config);
        tree.write("docs/ar.md", AR);
        for (path, text) in files {
            tree.write(path, text);
        }
        tree
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
        fs::write(path, text).expect("write a file");
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).expect("read a file")
    }

    fn run(&self, args: &[&str]) -> Output {
        run_in(&self.0, args)
    }

    /// `grund <args>` prints exactly `stdout`, writes nothing to stderr, and exits `code`.
    fn assert_prints(&self, args: &[&str], stdout: &str, code: i32) {
        let output = self.run(args);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            stdout,
            "stdout of grund {args:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            "",
            "grund {args:?}"
        );
        assert_eq!(output.status.code(), Some(code), "grund {args:?}");
    }

    /// `grund <args>` prints nothing to stdout, exactly `stderr` to stderr, and exits 2.
    fn assert_refuses(&self, args: &[&str], stderr: &str) {
        let output = self.run(args);
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            stderr,
            "grund {args:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "",
            "grund {args:?}"
        );
        assert_eq!(output.status.code(), Some(2), "grund {args:?}");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run grund")
}

/// A `README.md` under [`SELECTED`], checked: exactly `stdout`, exit `code`.
fn check_selected(name: &str, readme: &str, stdout: &str, code: i32) {
    let tree = Tree::new(name, SELECTED, &[("README.md", readme)]);
    tree.assert_prints(&["check"], stdout, code);
}

/// The control, green today and after: with no `fenced` key the fence stays skipped, its dangling
/// label passes, and the declaration only the diagram cites is reported unused.
#[test]
fn without_the_key_a_mermaid_fence_stays_skipped() {
    let tree = Tree::new("plain", PLAIN, &[("README.md", ISSUE)]);
    tree.assert_prints(
        &["check"],
        "README.md:9: error: unknown reference AR-nonexistent2\n\
         docs/ar.md:1: warning: declared but never cited: AR-gateway; cite it\n",
        1,
    );
}

/// The oracle, green today and after: the issue's text with its fence written as `<pre>` prints
/// exactly what the selected fence must, so the red test below pins the fence and nothing else.
#[test]
fn the_pre_twin_prints_what_a_selected_fence_must() {
    let tree = Tree::new("pre-twin", PLAIN, &[("README.md", ISSUE_PRE)]);
    tree.assert_prints(&["check"], ISSUE_CHECK, 1);
    tree.assert_prints(
        &["refs", "AR-gateway"],
        "README.md:5: \u{a7}AR-gateway\n",
        0,
    );
    tree.assert_prints(&["cover", "README.md", "--format", "json"], ISSUE_COVER, 0);
}

/// The issue's example: the label's dangling citation is reported at line 5 and column 54, the
/// diagram's citation of `AR-gateway` is a use `refs` lists, and the unused warning is gone
/// (today: `unknown key 'fenced'`; a build that only accepted the key would print the control).
#[test]
#[ignore = "red until #1364 reads a selected fence's citations; implement removes this"]
fn the_issue_example_reads_the_mermaid_fence() {
    let tree = Tree::new("issue", SELECTED, &[("README.md", ISSUE)]);
    tree.assert_prints(&["check"], ISSUE_CHECK, 1);
    tree.assert_prints(
        &["refs", "AR-gateway"],
        "README.md:5: \u{a7}AR-gateway\n",
        0,
    );
    tree.assert_prints(&["cover", "README.md", "--format", "json"], ISSUE_COVER, 0);
}

/// `Mermaid`, `{.mermaid}`, an empty info string, `text`, `toml` and `rust` are not the tag
/// `mermaid`, so their dangling citations stay unread, while the one `mermaid` fence is read and
/// keeps `AR-gateway` from being unused.
#[test]
#[ignore = "red until #1364 matches a fence's tag literally; implement removes this"]
fn only_the_literal_tag_is_selected() {
    let readme = "**Fences**\n\n```mermaid\ngateway[\"\u{a7}AR-gateway\"]\n```\n\n\
                  ```Mermaid\nx[\"\u{a7}AR-gone-capital\"]\n```\n\n\
                  ```{.mermaid}\nx[\"\u{a7}AR-gone-attribute\"]\n```\n\n\
                  ```\n\u{a7}AR-gone-empty\n```\n\n```text\n\u{a7}AR-gone-text\n```\n\n\
                  ```toml\nkey = \"\u{a7}AR-gone-toml\"\n```\n\n```rust\n/// \u{a7}AR-gone-rust\n```\n";
    check_selected("literal", readme, "success\n", 0);
}

/// The tag is the first word after any opener: words after it, spaces before it, tildes, a longer
/// backtick run holding a shorter one, and an indented opener each select the fence.
#[test]
#[ignore = "red until #1364 reads the tag after every fence opener; implement removes this"]
fn the_tag_is_the_first_word_after_any_opener() {
    let readme = "**Fences**\n\n\
                  ```mermaid title=\"Architecture\"\n\
                  gateway[\"\u{a7}AR-gateway\"] --> x[\"\u{a7}AR-gone-titled\"]\n```\n\n\
                  ```   mermaid\n\u{a7}AR-gone-spaced\n```\n\n\
                  ~~~mermaid\n\u{a7}AR-gone-tilde\n~~~\n\n\
                  ````mermaid\n```\n\u{a7}AR-gone-long\n````\n\n   ```mermaid\n\
                  \u{a7}AR-gone-indented\n   ```\n";
    check_selected(
        "openers",
        readme,
        "README.md:4: error: unknown reference AR-gone-titled\n\
         README.md:8: error: unknown reference AR-gone-spaced\n\
         README.md:12: error: unknown reference AR-gone-tilde\n\
         README.md:17: error: unknown reference AR-gone-long\n\
         README.md:21: error: unknown reference AR-gone-indented\n",
        1,
    );
}

/// A selected fence that never closes runs to end of file, and its citations stay live there; the
/// declaration-shaped line inside it declares nothing.
#[test]
#[ignore = "red until #1364 keeps an unclosed selected fence live; implement removes this"]
fn an_unclosed_selected_fence_stays_live_to_end_of_file() {
    let readme = "**Demo**\n\n```mermaid\ngateway[\"\u{a7}AR-gateway\"]\n\n\
                  \u{23} AR-later: Later\n\u{a7}AR-gone-eof\n";
    check_selected(
        "unclosed",
        readme,
        "README.md:7: error: unknown reference AR-gone-eof\n",
        1,
    );
}

/// A `mermaid` definition with a `%%` comment host does not read the fence: the Markdown reader
/// does, so the label line outside any `%%` comment is read too
/// (§FS-config-v2.schema.sources.fenced).
#[test]
#[ignore = "red until #1364 reads a selected fence with the Markdown reader; implement removes this"]
fn a_definition_sharing_the_tag_does_not_read_the_fence() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  languages = [\"markdown\", \"mermaid\"]\nfenced = [\"mermaid\"]\n\
                  [schema.sources.extensions]\nmmd = \"mermaid\"\n\
                  [schema.sources.definitions.mermaid]\nclass = \"source\"\n\
                  comment_prefixes = [\"%%\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";
    let readme = "**Demo**\n\n```mermaid\n%% \u{a7}AR-gone-comment\n\
                  gateway[\"\u{a7}AR-gateway\"] --> db[\"\u{a7}AR-gone-label\"]\n```\n";
    let tree = Tree::new("definition", config, &[("README.md", readme)]);
    tree.assert_prints(
        &["check"],
        "README.md:4: error: unknown reference AR-gone-comment\n\
         README.md:5: error: unknown reference AR-gone-label\n",
        1,
    );
}

/// `# AR-inside: Inside` in a selected fence declares nothing, so the prose citation of it is
/// unknown, while the fence's citation of `AR-gateway` is a use.
#[test]
#[ignore = "red until #1364 reads a selected fence's citations and nothing else; implement removes this"]
fn a_declaration_line_in_a_selected_fence_declares_nothing() {
    let readme = "**Demo**\n\n```mermaid\n# AR-inside: Inside\ngateway[\"\u{a7}AR-gateway\"]\n\
                  ```\n\nSee \u{a7}AR-inside.\n";
    check_selected(
        "declaration",
        readme,
        "README.md:8: error: unknown reference AR-inside\n",
        1,
    );
}

/// `## 1. Inside` in a selected fence in `AR-gateway`'s body is no section, so `AR-gateway.1` is
/// missing, while the fence's dangling citation is reported.
#[test]
#[ignore = "red until #1364 reads a selected fence's citations and nothing else; implement removes this"]
fn a_numbered_heading_in_a_selected_fence_is_no_section() {
    let tree = Tree::new(
        "section",
        SELECTED,
        &[
            ("README.md", "See \u{a7}AR-gateway.1.\n"),
            (
                "docs/ar.md",
                "# AR-gateway: Gateway\n\nThe lead.\n\n```mermaid\n## 1. Inside\n\
                 db[\"\u{a7}AR-gone\"]\n```\n",
            ),
        ],
    );
    tree.assert_prints(
        &["check"],
        "README.md:1: error: missing section AR-gateway.1\n\
         docs/ar.md:7: error: unknown reference AR-gone\n",
        1,
    );
}

/// `<§>AR-nonexistent` in a selected fence is an escape, not a citation, so the run is clean.
#[test]
#[ignore = "red until #1364 reads a selected fence's citations; implement removes this"]
fn an_escaped_id_in_a_selected_fence_is_no_citation() {
    let readme = "**Demo**\n\n```mermaid\n\
                  gateway[\"\u{a7}AR-gateway\"] --> x[\"<\u{a7}>AR-nonexistent\"]\n```\n";
    check_selected("escape", readme, "success\n", 0);
}

/// A `grund:fmt off` line in a selected fence opens no region: `fmt --write` still marks and links
/// the trigger after the fence, as it does with no `fenced` key, and leaves the fence's bytes.
#[test]
#[ignore = "red until #1364 keeps a selected fence's directives inert; implement removes this"]
fn a_fmt_directive_in_a_selected_fence_toggles_nothing() {
    let fence = "```mermaid\n<!-- grund:fmt off -->\ngateway[\"\u{a7}AR-gone\"]\n```\n";
    let readme = format!("**Demo**\n\n{fence}\nOutside cites \u{24}\u{24}AR-gateway.\n");
    let tree = Tree::new("directive", SELECTED, &[("README.md", &readme)]);
    tree.assert_prints(
        &["fmt", "--write"],
        "rewrote 1 line: 1 trigger marked, 1 link written\n  README.md (1)\n",
        0,
    );
    assert_eq!(
        tree.read("README.md"),
        format!(
            "**Demo**\n\n{fence}\n\
             Outside cites [\u{a7}AR-gateway](docs/ar.md#ar-gateway-gateway).\n"
        )
    );
    tree.assert_prints(
        &["check"],
        "README.md:5: error: unknown reference AR-gone\n",
        1,
    );
}

/// Under the canonical shorthand policy, `fmt --write` rewrites the shorthand and the trigger
/// after a selected fence and none of the fence's bytes, and `check` then reports the shorthand
/// left inside it, which a second `fmt --write` does not fix (§FS-citations.rewrite-exemption).
#[test]
#[ignore = "red until #1364 reads a selected fence's citations; implement removes this"]
fn fmt_leaves_a_selected_fence_byte_for_byte() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema]\n\
                  id_format = \"{kind}-{number}-{slug}\"\n[schema.sources]\nfenced = [\"mermaid\"]\n\
                  [schema.kinds.FS]\nfiles = [\"docs/fs.md\"]\n";
    let fence = "```mermaid\nflowchart LR\n  a[\"\u{a7}FS-042\"] --> b[\"\u{a7}FS-042-user-login\"] \
                 --> c[\"\u{24}\u{24}FS-042-user-login\"]\n```\n";
    let readme = format!(
        "**Demo**\n\n{fence}\nOutside cites \u{a7}FS-042 and \u{24}\u{24}FS-042-user-login.\n"
    );
    let tree = Tree::new(
        "fmt",
        config,
        &[
            ("docs/fs.md", "# FS-042-user-login: User login\n"),
            ("README.md", &readme),
        ],
    );
    tree.assert_prints(
        &["fmt", "--write"],
        "rewrote 1 line: 1 trigger marked, 1 shorthand expanded, 2 links written\n  README.md (1)\n",
        0,
    );
    let link = "[\u{a7}FS-042-user-login](docs/fs.md#fs-042-user-login-user-login)";
    let written = format!("**Demo**\n\n{fence}\nOutside cites {link} and {link}.\n");
    assert_eq!(tree.read("README.md"), written);
    tree.assert_prints(
        &["check"],
        "README.md:5: error: shorthand citation \u{a7}FS-042; write \u{a7}FS-042-user-login\n",
        1,
    );
    tree.assert_prints(&["fmt", "--write"], "rewrote 0 lines\n", 0);
    assert_eq!(tree.read("README.md"), written);
}

/// A tag outside `[A-Za-z0-9][A-Za-z0-9_+.#-]*` is refused at the `fenced` line.
#[test]
#[ignore = "red until #1364 reads and validates [schema.sources] fenced; implement removes this"]
fn a_malformed_fence_tag_is_refused_at_its_line() {
    for (case, tag) in [
        ("attribute", "{.mermaid}"),
        ("empty", ""),
        ("leading-hyphen", "-mermaid"),
        ("space", "mer maid"),
        ("bang", "mermaid!"),
    ] {
        let config = format!(
            "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
             fenced = [\"mermaid\", \"{tag}\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n"
        );
        let tree = Tree::new(&format!("malformed-{case}"), &config, &[]);
        tree.assert_refuses(
            &["check"],
            &format!(
                "error: grund.toml:4: invalid fence tag `{tag}` in [schema.sources] fenced \
                 (expected [A-Za-z0-9][A-Za-z0-9_+.#-]*)\n"
            ),
        );
    }
}

/// Every character the tag grammar admits loads, and `mermaid` and `Mermaid` are two tags.
#[test]
#[ignore = "red until #1364 reads and validates [schema.sources] fenced; implement removes this"]
fn tags_of_the_grammar_load_and_compare_case_sensitively() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  fenced = [\"mermaid\", \"Mermaid\", \"c++\", \"c#\", \"objective-c\", \"x.y\", \
                  \"a_b\", \"9p\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";
    let readme = "**Demo**\n\n```Mermaid\ngateway[\"\u{a7}AR-gateway\"]\n```\n";
    let tree = Tree::new("grammar", config, &[("README.md", readme)]);
    tree.assert_prints(&["check"], "success\n", 0);
}

/// A tag written twice is refused at the `fenced` line.
#[test]
#[ignore = "red until #1364 reads and validates [schema.sources] fenced; implement removes this"]
fn a_duplicate_fence_tag_is_refused_at_its_line() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  fenced = [\"mermaid\", \"dot\", \"mermaid\"]\n\
                  [schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";
    let tree = Tree::new("duplicate", config, &[]);
    tree.assert_refuses(
        &["check"],
        "error: grund.toml:4: fence tag `mermaid` is listed twice in [schema.sources] fenced\n",
    );
}

/// `config show` for [`PLAIN`] with `fenced` printed as `line`.
fn shown(line: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema]\nmarker = \"\u{a7}\"\n\
         shorthand = \"canonical\"\nid_format = \"{{kind}}-{{slug}}\"\n\
         slug_pattern = \"[a-z][a-z0-9-]*\"\nnumber_pattern = \"\\\\d+\"\n\
         section_separator = \".\"\nheading_depth = \"must\"\n\n[schema.sources]\n\
         languages = [\"markdown\", \"json\", \"rust\", \"c\", \"cpp\", \"java\", \"kotlin\", \
         \"scala\", \"swift\", \"javascript\", \"typescript\", \"csharp\", \"php\", \"python\", \
         \"go\", \"ruby\", \"sql\", \"lua\", \"haskell\", \"lisp\", \"scheme\", \"clojure\", \
         \"ada\"]\ndeclarations = \"doc-comments\"\nexclude = []\nrespect_gitignore = true\n\
         {line}\n\n[schema.notes.lines]\nmust = 3\n\n[schema.notes.columns]\nmust = 100\n\n\
         [schema.notes.text]\nmust = true\n\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n\n\
         [presentation]\ntrigger = \"\u{24}\u{24}\"\n\n[presentation.fmt]\nanchors = \"github\"\n\
         links = \"index\"\nexclude = []\n"
    )
}

/// `config show` prints `fenced` after `respect_gitignore`, `[]` where it is omitted and the
/// written list in its order, and what it prints loads back to the same output.
#[test]
#[ignore = "red until #1364 prints fenced in the effective config; implement removes this"]
fn config_show_prints_fenced_and_loads_it_back() {
    let plain = Tree::new("show-plain", PLAIN, &[]);
    plain.assert_prints(&["config", "show"], &shown("fenced = []"), 0);

    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  fenced = [\"mermaid\", \"dot\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";
    let written = Tree::new("show-written", config, &[]);
    let expected = shown("fenced = [\"mermaid\", \"dot\"]");
    written.assert_prints(&["config", "show"], &expected, 0);

    let reloaded = Tree::new("show-reloaded", &expected, &[]);
    reloaded.assert_prints(&["config", "show"], &expected, 0);
}

/// `config explain` documents the key like its siblings, and lists it after `respect_gitignore`
/// in the table's own entry.
#[test]
#[ignore = "red until #1364 documents fenced for config explain; implement removes this"]
fn config_explain_names_fenced() {
    let tree = Tree::new("explain", PLAIN, &[]);
    let output = tree.run(&["config", "explain", "[schema.sources] fenced"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[..4],
        [
            "grund.toml is read as version 2.",
            "",
            "[schema.sources] fenced",
            "Values: array of strings"
        ],
        "{stdout}"
    );
    assert_eq!(lines.len(), 5, "one description line: {stdout}");
    assert!(!lines[4].trim().is_empty(), "a description: {stdout}");

    let table = tree.run(&["config", "explain", "[schema.sources]"]);
    let table = String::from_utf8_lossy(&table.stdout);
    let rows: Vec<&str> = table.lines().collect();
    let gitignore = rows
        .iter()
        .position(|row| row.starts_with("  respect_gitignore (boolean): "))
        .unwrap_or_else(|| panic!("respect_gitignore row: {table}"));
    assert!(
        rows[gitignore + 1].starts_with("  fenced (array of strings): "),
        "fenced follows respect_gitignore: {table}"
    );
}

/// In a kind's index, a selected fence's line mentions what it cites, so the missing entry is
/// said with "appears"; the citation there is still no entry, and draws no not-a-link error.
#[test]
#[ignore = "red until #1364 looks at a selected fence in an index; implement removes this"]
fn a_selected_fence_in_an_index_mentions_but_lists_nothing() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  fenced = [\"mermaid\"]\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n";
    let tree = Tree::new(
        "index",
        config,
        &[
            (
                "docs/fs/FS-login.md",
                "# FS-login: User login\n\nIssues \u{a7}FS-token.\n",
            ),
            (
                "docs/fs/FS-token.md",
                "# FS-token: User token\n\nIssued after \u{a7}FS-login.\n",
            ),
            (
                "docs/fs/README.md",
                "# Index\n\n- [\u{a7}FS-token](FS-token.md#fs-token-user-token)\n\n\
                 ```mermaid\nflowchart LR\n  \
                 login[\"\u{a7}FS-login\"] --> token[\"\u{a7}FS-token\"]\n```\n",
            ),
        ],
    );
    tree.assert_prints(
        &["check"],
        "docs/fs/FS-login.md:1: error: FS-login appears in docs/fs/README.md but not as an \
         entry: an entry is a `\u{a7}`-marked Markdown link to the declaration — became an error \
         in grund 0.13.0\n",
        1,
    );
}

/// The marker bullet's last clause as today's block writes it.
const TODAY: &str = "or use a fenced code block.";

/// `init <args>` in `tree` prints nothing to stdout and exits 0; its report, on stderr.
fn init_reports(tree: &Tree, args: &[&str]) -> String {
    let output = tree.run(args);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "",
        "grund {args:?}"
    );
    let report = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "grund {args:?}; stderr: {report}"
    );
    report
}

/// `init --no-vcs` in a tree holding `config`: the report it prints and the `AGENTS.md` it writes.
fn init(name: &str, config: &str) -> (Tree, String, String) {
    let tree = Tree::new(name, config, &[]);
    let report = init_reports(&tree, &["init", "--no-vcs"]);
    let agents = tree.read("AGENTS.md");
    (tree, report, agents)
}

/// With tags selected, the block's marker bullet names only the fences they leave unread, the
/// tags in written order joined with "or", and changes nothing else, its version included; the
/// rewritten block is current for `init --check`.
#[test]
#[ignore = "red until #1364 renders the fence clause from fenced; implement removes this"]
fn init_names_only_the_unselected_fences() {
    let (_plain, plain_report, plain_agents) = init("init-plain", PLAIN);
    assert_eq!(plain_agents.matches(TODAY).count(), 1, "{plain_agents}");

    for (case, list, clause) in [
        (
            "one",
            "\"mermaid\"",
            "or use a fenced code block not tagged `mermaid`.",
        ),
        (
            "two",
            "\"mermaid\", \"dot\"",
            "or use a fenced code block not tagged `mermaid` or `dot`.",
        ),
        (
            "three",
            "\"mermaid\", \"dot\", \"plantuml\"",
            "or use a fenced code block not tagged `mermaid`, `dot`, or `plantuml`.",
        ),
    ] {
        let config = format!(
            "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
             fenced = [{list}]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n"
        );
        let (tree, report, agents) = init(&format!("init-{case}"), &config);
        assert_eq!(report, plain_report, "init reports the same in {case}");
        assert_eq!(agents, plain_agents.replace(TODAY, clause), "{case}");
        assert_eq!(
            init_reports(&tree, &["init", "--check", "--no-vcs"]),
            "exists AGENTS.md\nexists grund.toml\n",
            "{case}"
        );
    }
}

/// `fenced = []` selects nothing, so the block is today's, byte for byte.
#[test]
#[ignore = "red until #1364 reads [schema.sources] fenced; implement removes this"]
fn init_with_an_empty_fenced_writes_todays_block() {
    let (_plain, plain_report, plain_agents) = init("init-empty-plain", PLAIN);
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                  fenced = []\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";
    let (_empty, report, agents) = init("init-empty", config);
    assert_eq!(report, plain_report);
    assert_eq!(agents, plain_agents);
}
