//! `<subject> must cite at least N distinct <KIND>.` counts the different declarations a subject
//! cites (§FS-rules.3.2.1), so one source cited three times is one, and its
//! `citation-cardinality` row names every site it read, in text and JSON (§FS-rules.7.3,
//! §FS-rules.7.3.1). The plain sentence `… must cite at least 3 SRC.` keeps counting sites, with
//! #1366's bytes, and a sentence that holds `distinct` where the predicate does not admit it is
//! refused for that on both rule surfaces (§FS-rule-refusals.sentences,
//! §DF-distinct-target-counts). Issue #1367 is the report, and its reproducer is this fixture:
//! a country record cites one source three times, the plain rule passes, and the sentence that
//! says what the project means is refused as `unknown kind "distinct SRC"`.
//!
//! They were written before the sentence front end read `distinct`, as Rust tests rather than
//! goldens because a golden has no skip. The trees are built in the temp directory, so
//! run them with a `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The issue's project: countries, the sources they cite, and rules over them.
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"distinct-demo\"\n\
                      [schema]\nid_format = \"{kind}-{number}-{slug}\"\n\
                      [schema.kinds.COUNTRY]\nfolders = [\"docs/countries\"]\nindex = false\n\
                      [schema.kinds.SRC]\nfolders = [\"docs/sources\"]\nindex = false\n\
                      [schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\n\
                      form = \"rule\"\n[presentation]\nrules = \"root\"\n";

const C1: &str = "docs/countries/COUNTRY-001-rs.md";
const C2: &str = "docs/countries/COUNTRY-002-hr.md";
const C3: &str = "docs/countries/COUNTRY-003-si.md";

/// The issue's record: one source, cited at lines 5, 6 and 7 of its `truck-mass` chapter.
const SERBIA: (&str, &str) = (
    C1,
    "# COUNTRY-001-rs: Serbia\n\n## truck-mass: Truck mass\n\n\
     Maximum laden mass is 40 t (\u{a7}SRC-220-rs-vehicle-rulebook), 44 t for\n\
     combined transport (\u{a7}SRC-220-rs-vehicle-rulebook), and 11.5 t per\n\
     drive axle (\u{a7}SRC-220-rs-vehicle-rulebook).\n",
);
/// Two sources, one of them at its numbered section: two sites and two declarations.
const CROATIA: (&str, &str) = (
    C2,
    "# COUNTRY-002-hr: Croatia\n\n## truck-mass: Truck mass\n\n\
     Maximum laden mass is 40 t (\u{a7}SRC-221-rs-road-act), and 11.5 t per\n\
     drive axle (\u{a7}SRC-222-rs-ministry-notice.1).\n",
);
/// Three different sources: it satisfies both rules.
const SLOVENIA: (&str, &str) = (
    C3,
    "# COUNTRY-003-si: Slovenia\n\n## truck-mass: Truck mass\n\n\
     Maximum laden mass is 40 t (\u{a7}SRC-220-rs-vehicle-rulebook), 44 t for\n\
     combined transport (\u{a7}SRC-221-rs-road-act), and 11.5 t per\n\
     drive axle (\u{a7}SRC-222-rs-ministry-notice).\n",
);

const DISTINCT: &str = "The truck-mass chapter of each COUNTRY must cite at least 3 distinct SRC.";
const PLAIN: &str = "The truck-mass chapter of each COUNTRY must cite at least 3 SRC.";

/// `RULE-001` is the sentence the project means; `RULE-002` is the issue's own sentence.
const RULE_DISTINCT: (&str, &str) = ("RULE-001-country-truck-mass", DISTINCT);
const RULE_PLAIN: (&str, &str) = ("RULE-002-country-truck-mass-sites", PLAIN);

const REMEDY: &str = "add or remove those citations until the count fits";

/// `; counted at …` over `(line, citation as written)` entries of one file (§FS-rules.7.3.1).
fn clause(path: &str, counted: &[(usize, &str)]) -> String {
    let entries: Vec<String> = counted
        .iter()
        .map(|(line, written)| format!("{path}:{line} ({written})"))
        .collect();
    format!("; counted at {}", entries.join(", "))
}

const C1_COUNTED: &[(usize, &str)] = &[
    (5, "SRC-220-rs-vehicle-rulebook"),
    (6, "SRC-220-rs-vehicle-rulebook"),
    (7, "SRC-220-rs-vehicle-rulebook"),
];
const C2_COUNTED: &[(usize, &str)] = &[
    (5, "SRC-221-rs-road-act"),
    (6, "SRC-222-rs-ministry-notice.1"),
];

/// A distinct row's message, by the rule or `--rule` that authored it (§FS-rules.7.3).
fn distinct_message(
    subject: &str,
    n: usize,
    authority: &str,
    path: &str,
    counted: &[(usize, &str)],
) -> String {
    format!(
        "{subject}.truck-mass cites {n} distinct SRC; {authority} requires at least 3; {REMEDY}{}",
        clause(path, counted)
    )
}

/// The plain row of `RULE-002` on Croatia, exactly as #1366 prints it.
fn plain_message() -> String {
    format!(
        "COUNTRY-002-hr.truck-mass cites SRC 2 times; RULE-002-country-truck-mass-sites requires \
         at least 3; {REMEDY}{}",
        clause(C2, C2_COUNTED)
    )
}

/// One text row at a chapter title on line 3.
fn row(path: &str, message: &str) -> String {
    format!("{path}:3: error: {message}")
}

/// One NDJSON record of a count row at line 3 (§FS-errors.5.1).
fn record(path: &str, message: &str, lines: &[usize], rule: &str) -> String {
    let sites: Vec<String> = lines
        .iter()
        .map(|line| format!("{{\"path\":\"{path}\",\"line\":{line}}}"))
        .collect();
    format!(
        "{{\"severity\":\"error\",\"path\":\"{path}\",\"line\":3,\"code\":\"citation-cardinality\",\
         \"message\":\"{message}\",\"sites\":[{}],\"authority\":[\"{rule}\"]}}",
        sites.join(",")
    )
}

/// A scratch copy of the issue's project with the given countries and rules, removed on drop;
/// `docs/overview.md` cites every country and rule so no row but the rules' is printed.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, countries: &[(&str, &str)], rules: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1367-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        let tree = Self(root);
        tree.write("grund.toml", CONFIG);
        for slug in ["220-rs-vehicle-rulebook", "221-rs-road-act"] {
            let text = format!("# SRC-{slug}: Source {slug}\n\nA source.\n");
            tree.write(&format!("docs/sources/SRC-{slug}.md"), &text);
        }
        tree.write(
            "docs/sources/SRC-222-rs-ministry-notice.md",
            "# SRC-222-rs-ministry-notice: Source 222\n\nA source.\n\n## 1. Axles\n\nA part.\n",
        );
        let mut cited = Vec::new();
        for (path, text) in countries {
            tree.write(path, text);
            cited.push(format!(
                "\u{a7}{}",
                &path["docs/countries/".len()..path.len() - 3]
            ));
        }
        for (id, sentence) in rules {
            let text = format!("# {id}: {sentence}\n\nEach figure stands on its sources.\n");
            tree.write(&format!("docs/rules/{id}.md"), &text);
            cited.push(format!("\u{a7}{id}"));
        }
        tree.write(
            "docs/overview.md",
            &format!("Records: {}.\n", cited.join(", ")),
        );
        tree
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
        fs::write(path, text).expect("write a file");
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    /// `grund <args>` prints exactly the lines `stdout` holds, in any order, writes `stderr`, and
    /// exits `code`. Two rows at one chapter title are compared as a set: their order is the
    /// report's, not this change's.
    fn assert_prints(&self, args: &[&str], stdout: &[String], stderr: &str, code: i32) {
        let output = self.run(args);
        let printed = String::from_utf8_lossy(&output.stdout);
        let mut lines: Vec<&str> = printed.lines().collect();
        let mut expected: Vec<&str> = stdout.iter().map(String::as_str).collect();
        lines.sort_unstable();
        expected.sort_unstable();
        assert_eq!(
            lines,
            expected,
            "stdout of grund {args:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            stderr,
            "grund {args:?}"
        );
        assert_eq!(output.status.code(), Some(code), "grund {args:?}");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The control: the issue's own sentence counts sites, so Serbia's three
/// citations of one source pass it and Croatia's two fail it, with #1366's row and list
/// (§FS-rules.3.2, §FS-rules.7.3.1). It proves the fixture sound, so a red test below is about
/// `distinct` alone.
#[test]
fn the_plain_count_keeps_its_verdict_and_bytes() {
    let tree = Tree::new("plain", &[SERBIA, CROATIA, SLOVENIA], &[RULE_PLAIN]);
    tree.assert_prints(&["check"], &[row(C2, &plain_message())], "", 1);
}

/// The issue's whole run, with both sentences: Serbia's one source is one distinct `SRC`, its row
/// lists all three citations, so the list adds up to the sites and not the count; Croatia fails
/// both rules, one row each; Slovenia passes both. The plain row keeps #1366's bytes.
#[test]
fn one_source_cited_three_times_is_one_distinct_source() {
    let tree = Tree::new(
        "text",
        &[SERBIA, CROATIA, SLOVENIA],
        &[RULE_DISTINCT, RULE_PLAIN],
    );
    let (rule, _) = RULE_DISTINCT;
    let stdout = [
        row(
            C1,
            &distinct_message("COUNTRY-001-rs", 1, rule, C1, C1_COUNTED),
        ),
        row(
            C2,
            &distinct_message("COUNTRY-002-hr", 2, rule, C2, C2_COUNTED),
        ),
        row(C2, &plain_message()),
    ];
    tree.assert_prints(&["check"], &stdout, "", 1);
}

/// The same rows in JSON: `sites` holds every counted line, and `authority` the one rule.
#[test]
fn the_distinct_row_fills_sites_in_json() {
    let tree = Tree::new(
        "json",
        &[SERBIA, CROATIA, SLOVENIA],
        &[RULE_DISTINCT, RULE_PLAIN],
    );
    let (distinct, _) = RULE_DISTINCT;
    let (plain, _) = RULE_PLAIN;
    let stdout = [
        record(
            C1,
            &distinct_message("COUNTRY-001-rs", 1, distinct, C1, C1_COUNTED),
            &[5, 6, 7],
            distinct,
        ),
        record(
            C2,
            &distinct_message("COUNTRY-002-hr", 2, distinct, C2, C2_COUNTED),
            &[5, 6],
            distinct,
        ),
        record(C2, &plain_message(), &[5, 6], plain),
    ];
    tree.assert_prints(&["check", "--format", "json"], &stdout, "", 1);
}

/// Three different sources satisfy the distinct rule, and its run succeeds.
#[test]
fn three_different_sources_satisfy_the_distinct_rule() {
    let tree = Tree::new("pass", &[SLOVENIA], &[RULE_DISTINCT]);
    tree.assert_prints(&["check"], &["success".to_string()], "", 0);
}

/// The issue's trial sentence on `check --rule … --only-rule`: today exit 2 and
/// `unknown kind "distinct SRC"`; after, the rows of the sentence, authored by `--rule`.
#[test]
fn the_issue_trial_sentence_reports_each_record() {
    let tree = Tree::new("trial", &[SERBIA, CROATIA, SLOVENIA], &[RULE_PLAIN]);
    let stdout = [
        row(
            C1,
            &distinct_message("COUNTRY-001-rs", 1, "--rule", C1, C1_COUNTED),
        ),
        row(
            C2,
            &distinct_message("COUNTRY-002-hr", 2, "--rule", C2, C2_COUNTED),
        ),
    ];
    tree.assert_prints(
        &["check", "--rule", DISTINCT, "--only-rule"],
        &stdout,
        "",
        1,
    );
}

/// §FS-rule-refusals.sentences, §DF-distinct-target-counts: a `distinct` the predicate does not
/// admit is refused for that, with its form, and a later unknown kind is refused under its own
/// name, on both surfaces: a configured rule's `invalid-rule` row after
/// `<RULE> is not a valid rule: `, and `check --rule` on stderr.
#[test]
fn a_misplaced_distinct_is_refused_for_what_it_is_on_both_surfaces() {
    let redundant = "The truck-mass chapter of each COUNTRY must cite at least one distinct SRC.";
    let unknown =
        "The truck-mass chapter of each COUNTRY must cite at least 3 distinct SRC or DOCX.";
    let rules = [
        ("RULE-003-country-one-source", redundant),
        ("RULE-004-country-docs", unknown),
    ];
    let tree = Tree::new("refused", &[SLOVENIA], &rules);
    let one = "\"distinct\" is redundant after \"at least one\"; accepted form: The truck-mass \
               chapter of each COUNTRY must cite at least one SRC.";
    let stdout = [
        format!(
            "docs/rules/RULE-003-country-one-source.md:1: error: RULE-003-country-one-source is \
             not a valid rule: {one}"
        ),
        "docs/rules/RULE-004-country-docs.md:1: error: RULE-004-country-docs is not a valid \
         rule: unknown kind \"DOCX\""
            .to_string(),
    ];
    tree.assert_prints(&["check"], &stdout, "", 1);
    tree.assert_prints(
        &["check", "--rule", redundant],
        &[],
        &format!("error: {one}\n"),
        2,
    );
    let known = "error: unknown kind \"DOCX\"\nknown kinds: COUNTRY, SRC, RULE\n";
    tree.assert_prints(&["check", "--rule", unknown], &[], known, 2);
}
