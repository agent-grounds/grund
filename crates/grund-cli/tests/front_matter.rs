//! Black-box: a Markdown file's front matter is a region of its own. No line of it is a
//! heading, its citations are live and belong to the file's first declaration, and `fmt`
//! rewrites nothing in it (§FS-check.1.1.5.2, §FS-check.1.1.5.2.1). Ports the #1046 metric and
//! policy bundle and its reproducer.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 2
project_name = \"fm\"

[schema.kinds.METRIC]
files = [\"bundle/metrics/net-revenue.md\"]

[schema.kinds.POL]
files = [\"bundle/policies/revenue-recognition.md\"]

[schema.kinds.page]
citable = false
files = [\"README.md\"]
";

const POLICY: &str =
    "# POL-revenue-recognition: Revenue is recognised in the month of the stay\n\nA policy.\n";

const README: &str = "# Readme\n\nThe metric is §METRIC-net-revenue.\n";

const BODY: &str = "
# METRIC-net-revenue: Net revenue, monthly

Gross bookings less refunds and credits, recognised in the month of the stay.
";

/// The issue's metric, YAML front matter citing the policy, with no comment line.
fn cited() -> String {
    format!(
        "---\ntype: Metric\ntitle: Net revenue, monthly\nsources:\n  - id: rev-policy\n    \
         resource: §POL-revenue-recognition\n---\n{BODY}"
    )
}

const METRIC: &str = "bundle/metrics/net-revenue.md";

struct Fixture(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Fixture {
    /// The #1046 bundle with `metric` as `bundle/metrics/net-revenue.md`.
    fn new(metric: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1046-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [
            ("grund.toml", CONFIG),
            ("bundle/policies/revenue-recognition.md", POLICY),
            ("README.md", README),
            (METRIC, metric),
        ] {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    fn write(&self, path: &str, contents: &str) {
        fs::write(self.0.join(path), contents).unwrap();
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }

    fn run(&self, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    /// Run `grund args…`, require exit `code`, and return stdout.
    fn expect(&self, code: i32, args: &[&str]) -> String {
        let run = self.run(args);
        assert_eq!(
            run.code,
            Some(code),
            "grund {} exited {:?}: stdout:\n{}\nstderr:\n{}",
            args.join(" "),
            run.code,
            run.stdout,
            run.stderr
        );
        run.stdout
    }

    /// The `(id, line)` of every declaration `grund list --format json` reports.
    fn declarations(&self) -> Vec<(String, u64)> {
        self.expect(0, &["list", "--format", "json"])
            .lines()
            .map(|line| {
                let row: serde_json::Value = serde_json::from_str(line).expect("one JSON row");
                (
                    row["id"].as_str().unwrap().to_owned(),
                    row["line"].as_u64().unwrap(),
                )
            })
            .collect()
    }

    /// The `refs --format json` row of the citation of `id` in `path`.
    fn ref_row(&self, id: &str, path: &str) -> serde_json::Value {
        let rows = self.expect(0, &["refs", id, "--format", "json"]);
        rows.lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("one JSON row"))
            .find(|row| row["path"] == path)
            .unwrap_or_else(|| panic!("no citation of {id} in {path}:\n{rows}"))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn ids(declarations: &[(String, u64)]) -> Vec<&str> {
    declarations.iter().map(|(id, _)| id.as_str()).collect()
}

/// A YAML comment `# METRIC-gross: …` declares nothing, so the bundle checks clean
/// (§FS-check.1.1.5.2, §FS-declarations.requirements.mentions).
#[test]
fn a_yaml_comment_declares_nothing() {
    let fx = Fixture::new(&format!(
        "---\ntype: Metric\ntitle: Net revenue, monthly\n\
         # METRIC-gross: the gross figure is a separate metric\nsources:\n  - id: rev-policy\n    \
         resource: §POL-revenue-recognition\n---\n{BODY}"
    ));
    let declarations = fx.declarations();
    assert!(
        declarations.contains(&("METRIC-net-revenue".to_owned(), 10)),
        "{declarations:?}"
    );
    assert!(
        !ids(&declarations).contains(&"METRIC-gross"),
        "{declarations:?}"
    );
    assert_eq!(fx.expect(0, &["check"]), "success\n");
}

/// A comment that repeats the file's own ID is no second declaration of it.
#[test]
fn a_comment_repeating_the_files_own_id_is_no_duplicate() {
    let fx = Fixture::new(&format!(
        "---\ntype: Metric\n# METRIC-net-revenue: keep this title in step with the heading\n\
         title: Net revenue, monthly\n---\n{BODY}"
    ));
    let declarations = fx.declarations();
    let metric: Vec<_> = declarations
        .iter()
        .filter(|(id, _)| id == "METRIC-net-revenue")
        .collect();
    assert_eq!(
        metric,
        [&("METRIC-net-revenue".to_owned(), 7)],
        "{declarations:?}"
    );
    let run = fx.run(&["check"]);
    assert_eq!(run.code, Some(0), "{}{}", run.stdout, run.stderr);
    assert!(
        !run.stdout.contains("duplicate declaration"),
        "{}",
        run.stdout
    );
}

/// TOML front matter, `+++` as Hugo and Zola write it, reads the same way.
#[test]
fn a_toml_block_declares_nothing() {
    let fx = Fixture::new(&format!(
        "+++\ntitle = \"Net revenue, monthly\"\n\
         # METRIC-gross: the gross figure is a separate metric\n\
         policy = \"§POL-revenue-recognition\"\n+++\n{BODY}"
    ));
    let declarations = fx.declarations();
    assert!(
        !ids(&declarations).contains(&"METRIC-gross"),
        "{declarations:?}"
    );
    assert!(
        declarations.contains(&("METRIC-net-revenue".to_owned(), 7)),
        "{declarations:?}"
    );
    let row = fx.ref_row("POL-revenue-recognition", METRIC);
    assert_eq!(row["enclosing_declaration"], "METRIC-net-revenue", "{row}");
}

/// A `---` block closes at `...` as Pandoc allows, and `...` does not close a `+++` block.
#[test]
fn three_dots_close_a_yaml_block_only() {
    let fx = Fixture::new(&format!(
        "---\ntype: Metric\n# METRIC-gross: the gross figure is a separate metric\n...\n{BODY}"
    ));
    assert!(!ids(&fx.declarations()).contains(&"METRIC-gross"));

    // `...` in a `+++` block is content, so the block closes at the second `+++`.
    fx.write(
        METRIC,
        &format!("+++\ntitle = \"x\"\n...\n# METRIC-gross: still inside\n+++\n{BODY}"),
    );
    let declarations = fx.declarations();
    assert!(
        !ids(&declarations).contains(&"METRIC-gross"),
        "{declarations:?}"
    );
    assert!(
        declarations.contains(&("METRIC-net-revenue".to_owned(), 7)),
        "{declarations:?}"
    );
}

/// Guards that hold today and must keep holding: an unclosed block, a `---` that is not on
/// line 1, a `---` closed by `+++`, and a first line led by a byte-order mark are no front
/// matter, so a heading in them still declares (§FS-check.1.1.5.2).
#[test]
fn what_is_not_front_matter_reads_as_today() {
    let gross = "# METRIC-gross: the gross figure is a separate metric\n";
    for (case, metric) in [
        ("unclosed", format!("---\ntype: Metric\n{gross}{BODY}")),
        (
            "not on line 1",
            format!("\n---\ntype: Metric\n{gross}---\n{BODY}"),
        ),
        (
            "mismatched closer",
            format!("---\ntype: Metric\n{gross}+++\n{BODY}"),
        ),
        (
            "byte-order mark",
            format!("\u{feff}---\ntype: Metric\n{gross}---\n{BODY}"),
        ),
        (
            "indented opener",
            format!(" ---\ntype: Metric\n{gross}---\n{BODY}"),
        ),
    ] {
        let fx = Fixture::new(&metric);
        let declarations = fx.declarations();
        assert!(
            ids(&declarations).contains(&"METRIC-gross"),
            "{case}: {declarations:?}"
        );
    }
}

/// The front-matter citation belongs to the file's first declaration, under no section, so
/// `refs --from` lists it and a `must cite` rule sees it (§FS-check.1.1.5.2.1).
#[test]
fn a_front_matter_citation_belongs_to_the_first_declaration() {
    let fx = Fixture::new(&cited());
    let row = fx.ref_row("POL-revenue-recognition", METRIC);
    assert_eq!(row["line"], 6, "{row}");
    assert_eq!(row["column"], 15, "{row}");
    assert_eq!(row["enclosing_declaration"], "METRIC-net-revenue", "{row}");
    assert!(row["enclosing_section"].is_null(), "{row}");

    let from = fx.expect(0, &["refs", "--from", "METRIC-net-revenue"]);
    assert!(
        from.contains(&format!("{METRIC}:6: §POL-revenue-recognition")),
        "refs --from:\n{from}"
    );
    fx.expect(
        0,
        &[
            "check",
            "--rule",
            "Each METRIC must cite at least one POL.",
            "--only-rule",
        ],
    );

    let cover = fx.expect(0, &["cover", METRIC, "--format", "json"]);
    let cover: serde_json::Value = serde_json::from_str(cover.trim()).expect("one cover row");
    assert_eq!(
        cover["citations"][0]["enclosing_declaration"], "METRIC-net-revenue",
        "{cover}"
    );
}

/// The owner is the first *declaration*, not the first heading.
#[test]
fn the_owner_is_the_first_declaration_not_the_first_heading() {
    let fx = Fixture::new(
        "---\nsee: §POL-revenue-recognition\n---\n\n# Metrics\n\nThe bundle's metrics.\n\n\
         ## METRIC-net-revenue: Net revenue, monthly\n\nGross bookings.\n",
    );
    let row = fx.ref_row("POL-revenue-recognition", METRIC);
    assert_eq!(row["line"], 2, "{row}");
    assert_eq!(row["enclosing_declaration"], "METRIC-net-revenue", "{row}");
    assert!(row["enclosing_section"].is_null(), "{row}");
}

/// In a file that declares nothing, a front-matter citation has no owner; it still resolves
/// and counts as a use. Holds today and must keep holding.
#[test]
fn a_file_with_no_declaration_owns_nothing() {
    let fx = Fixture::new(&cited());
    fx.write(
        "README.md",
        "---\ntitle: Readme\nsee: §POL-revenue-recognition\n---\n\n# Readme\n\nThe metric is \
         §METRIC-net-revenue.\n",
    );
    let row = fx.ref_row("POL-revenue-recognition", "README.md");
    assert_eq!(row["line"], 3, "{row}");
    assert!(row["enclosing_declaration"].is_null(), "{row}");
    assert!(row["enclosing_section"].is_null(), "{row}");
}

/// `fmt --check` reports nothing in front matter, and `fmt --write` leaves it byte-identical
/// while it still links the README's citation (§FS-citations.zones).
#[test]
fn fmt_never_rewrites_front_matter() {
    let fx = Fixture::new(&cited());
    let check = fx.run(&["fmt", "--check"]);
    let reported = format!("{}{}", check.stdout, check.stderr);
    assert!(
        reported.contains("README.md:3:"),
        "fmt --check:\n{reported}"
    );
    assert!(!reported.contains(METRIC), "fmt --check:\n{reported}");

    fx.expect(0, &["fmt", "--write"]);
    assert_eq!(fx.read(METRIC), cited());
    assert!(fx.read("README.md").contains("[§METRIC-net-revenue]("));
}

/// `cover --lines` gives every front-matter line, delimiters included, to the first
/// declaration; the blank line under the closing delimiter stays unowned (§FS-cover.6.2).
#[test]
fn cover_lines_gives_front_matter_to_the_first_declaration() {
    let fx = Fixture::new(&cited());
    assert_eq!(
        fx.expect(0, &["cover", METRIC, "--lines", "1-9"]),
        format!(
            "{METRIC}:1-9\n  1-7  METRIC-net-revenue\n  8  (no owner)\n  9  METRIC-net-revenue\n"
        )
    );
}

/// A local shorthand in front matter is owned, so it is reported with the form to write, but
/// `check` does not offer `grund fmt --write`, which would rewrite nothing there
/// (§FS-check.3.24.1).
#[test]
fn a_front_matter_shorthand_errors_without_the_fmt_offer() {
    let fx = Fixture::new(
        "---\ntype: Metric\nsee: \u{a7}2\n---\n\n# METRIC-net-revenue: Net revenue, monthly\n\n\
         ## 2. Policy\n\nCites §POL-revenue-recognition.\n",
    );
    let run = fx.run(&["check"]);
    assert_eq!(run.code, Some(1), "{}{}", run.stdout, run.stderr);
    let line = run
        .stdout
        .lines()
        .find(|line| line.starts_with(&format!("{METRIC}:3:")))
        .unwrap_or_else(|| panic!("no finding at {METRIC}:3:\n{}", run.stdout));
    assert!(
        line.contains("local section citation \u{a7}2; write §METRIC-net-revenue.2"),
        "{line}"
    );
    assert!(!line.contains("grund fmt --write"), "{line}");
}
