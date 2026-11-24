//! A `citation-cardinality` or `uncited-unit` row whose count is above zero names every citation
//! it counted, `; counted at <path>:<line> (<citation as written>), …` after the whole of the
//! message it printed before, and JSON `sites` carries each distinct `{path, line}` in the same
//! order (§FS-rules.7.3, §FS-rules.7.4, §FS-rules.7.6, §FS-errors.5.1). A zero
//! count keeps its bytes and `"sites":null`. A run narrowed to a counted citing file reports the
//! row whole, at its subject (§FS-check.1.3.6), and on a file with two names the identity
//! clause stays before the list (§FS-scan.identity.findings.content). Issue #1366 is the report:
//! a price record held to one item cites it and then the item's own section, the count reads 2,
//! and the row named neither site.
//!
//! They were written before the fix, as tests rather than goldens because a golden has no skip;
//! the golden case `check-rules-count-names-sites` pins the issue's own fixture. The trees are
//! built in the temp directory, so run them with a `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The issue's project: prices, the items they price, and five rules over them.
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"grund-1366\"\n\
                      [schema]\nid_format = \"{kind}-{number}-{slug}\"\n\
                      [schema.kinds.PRICE]\nfolders = [\"docs/prices\"]\nindex = false\n\
                      [schema.kinds.INPUT]\nfolders = [\"docs/inputs\"]\nindex = false\n\
                      [schema.kinds.LABOR]\nfolders = [\"docs/labor\"]\nindex = false\n\
                      [schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\n\
                      form = \"rule\"\n[presentation]\nrules = \"root\"\n";

/// Cites its item at line 7 and the item's own numbered section at line 12 (§FS-rules.5.1).
const PRICE_001: &str = "# PRICE-001-diesel-retail: Diesel, retail\n\n\
                         The price of a litre of diesel at the pump.\n\n\
                         - Unit: litre\n- Currency: RSD\n- Item: \u{a7}INPUT-001-diesel\n\n\
                         The figure is taken weekly; see how it is sourced\n\
                         in the item itself.\n\n\
                         Observed range: 190 to 210, from \u{a7}INPUT-001-diesel.1.\n";

/// The fixture's files under the root, beside `grund.toml`.
const FILES: &[(&str, &str)] = &[
    (
        "docs/inputs/INPUT-001-diesel.md",
        "# INPUT-001-diesel: Diesel fuel\n\nRetail diesel, per litre.\n\n\
         ## 1. Source\n\nPublished pump prices.\n",
    ),
    (
        "docs/labor/LABOR-001-driver.md",
        "# LABOR-001-driver: Driver hour\n\nOne hour of a licensed driver.\n",
    ),
    (
        "docs/labor/LABOR-002-mechanic.md",
        "# LABOR-002-mechanic: Mechanic hour\n\nOne hour of a mechanic.\n",
    ),
    (
        "docs/overview.md",
        "Prices: \u{a7}PRICE-001-diesel-retail, \u{a7}PRICE-002-haul, \u{a7}PRICE-003-driver,\n\
         held to \u{a7}RULE-001-price-item, \u{a7}RULE-002-input-use, \u{a7}RULE-003-labor-used\n\
         \u{a7}RULE-004-input-once and \u{a7}RULE-005-driver-once. \
         Labor: \u{a7}LABOR-002-mechanic.\n",
    ),
    ("docs/prices/PRICE-001-diesel-retail.md", PRICE_001),
    (
        "docs/prices/PRICE-002-haul.md",
        "# PRICE-002-haul: Haul, per hour\n\nA truck hour.\n\n\
         - Fuel: \u{a7}INPUT-001-diesel\n\n\n- Driver: \u{a7}LABOR-001-driver\n",
    ),
    (
        "docs/prices/PRICE-003-driver.md",
        "# PRICE-003-driver: Driver, per hour\n\n- Item: \u{a7}LABOR-001-driver\n",
    ),
    (
        "docs/rules/RULE-001-price-item.md",
        "# RULE-001-price-item: Each PRICE must cite exactly one INPUT or LABOR.\n\n\
         Every price is the price of one priced item.\n",
    ),
    (
        "docs/rules/RULE-002-input-use.md",
        "# RULE-002-input-use: Each INPUT must be cited by at most 2 PRICE.\n\n\
         An input feeds at most two prices.\n",
    ),
    (
        "docs/rules/RULE-003-labor-used.md",
        "# RULE-003-labor-used: Each LABOR must be cited by at least one PRICE.\n\n\
         Every labor item is priced somewhere.\n",
    ),
    (
        "docs/rules/RULE-004-input-once.md",
        "# RULE-004-input-once: Each PRICE must cite each INPUT at most once.\n\n\
         A price names its input once.\n",
    ),
    (
        "docs/rules/RULE-005-driver-once.md",
        "# RULE-005-driver-once: Each LABOR must be cited by at most 1 PRICE.\n\n\
         A labor item is priced once.\n",
    ),
];

// The messages these rows printed before #1366, each a verbatim prefix of the row after it
// (§FS-rules.7.3: the clause is appended after the whole message).
const INPUT_001: &str = "INPUT-001-diesel is cited by PRICE 3 times; RULE-002-input-use \
                         requires at most 2; add or remove the citations of it until the count fits";
const LABOR_001: &str = "LABOR-001-driver is cited by PRICE 2 times; RULE-005-driver-once \
                         requires at most 1; add or remove the citations of it until the count fits";
const LABOR_002: &str = "LABOR-002-mechanic is cited by PRICE 0 times; RULE-003-labor-used \
                         requires at least one; add or remove the citations of it until the count \
                         fits";
const P1_ONE_ITEM: &str = "PRICE-001-diesel-retail cites INPUT or LABOR 2 times; \
                           RULE-001-price-item requires exactly one; add or remove those \
                           citations until the count fits";
const P1_INPUT_ONCE: &str = "PRICE-001-diesel-retail cites INPUT-001-diesel 2 times; \
                             RULE-004-input-once requires at most 1; add or remove those citations \
                             until the count fits";
const P2_ONE_ITEM: &str = "PRICE-002-haul cites INPUT or LABOR 2 times; RULE-001-price-item \
                           requires exactly one; add or remove those citations until the count fits";

/// Each counted citation of a row, `(path, line, citation as written)`, in the order the clause
/// lists them: by path, then line, then column (§FS-rules.7.3).
type Counted<'a> = &'a [(&'a str, usize, &'a str)];

const P1: &str = "docs/prices/PRICE-001-diesel-retail.md";
const P2: &str = "docs/prices/PRICE-002-haul.md";
const P3: &str = "docs/prices/PRICE-003-driver.md";

/// Both of PRICE-001's citations, the section citation at line 12 among them.
const P1_COUNTED: Counted = &[(P1, 7, "INPUT-001-diesel"), (P1, 12, "INPUT-001-diesel.1")];
/// PRICE-002's over-count across two kinds.
const P2_COUNTED: Counted = &[(P2, 5, "INPUT-001-diesel"), (P2, 8, "LABOR-001-driver")];
/// INPUT-001's inbound citations, written in two other files.
const INPUT_COUNTED: Counted = &[
    (P1, 7, "INPUT-001-diesel"),
    (P1, 12, "INPUT-001-diesel.1"),
    (P2, 5, "INPUT-001-diesel"),
];
/// LABOR-001's inbound citations.
const LABOR_COUNTED: Counted = &[(P2, 8, "LABOR-001-driver"), (P3, 3, "LABOR-001-driver")];

/// `; counted at <path>:<line> (<citation>), …`, every path spelled from `base` the way the
/// report spells it (§FS-errors.3): `""` is the root, and a base is a directory every counted
/// citation here is written under.
fn clause(counted: Counted, base: &str) -> String {
    let entries: Vec<String> = counted
        .iter()
        .map(|(path, line, written)| {
            let path = match base {
                "" => path,
                base => path
                    .strip_prefix(&format!("{base}/"))
                    .expect("a counted citation under the base"),
            };
            format!("{path}:{line} ({written})")
        })
        .collect();
    format!("; counted at {}", entries.join(", "))
}

/// One text row: `<path>:1: error: <message>`.
fn row(path: &str, message: &str) -> String {
    format!("{path}:1: error: {message}\n")
}

/// One NDJSON record, keys in the published order (§FS-errors.5.1); `sites` empty is `null`.
fn record(code: &str, path: &str, message: &str, sites: &[(&str, usize)], rule: &str) -> String {
    let sites = if sites.is_empty() {
        "null".to_string()
    } else {
        let pairs: Vec<String> = sites
            .iter()
            .map(|(path, line)| format!("{{\"path\":\"{path}\",\"line\":{line}}}"))
            .collect();
        format!("[{}]", pairs.join(","))
    };
    format!(
        "{{\"severity\":\"error\",\"path\":\"{path}\",\"line\":1,\"code\":\"{code}\",\
         \"message\":\"{message}\",\"sites\":{sites},\"authority\":[\"{rule}\"]}}\n"
    )
}

/// Each distinct `(path, line)` of `counted`, in its order: two citations on one line are one
/// site (§FS-rules.7.3).
fn sites_of<'a>(counted: Counted<'a>) -> Vec<(&'a str, usize)> {
    let mut sites: Vec<(&str, usize)> = Vec::new();
    for (path, line, _) in counted {
        if !sites.contains(&(*path, *line)) {
            sites.push((*path, *line));
        }
    }
    sites
}

/// A scratch copy of the issue's project, removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1366-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        let tree = Self(root);
        tree.write("grund.toml", CONFIG);
        for (path, text) in FILES {
            tree.write(path, text);
        }
        tree
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
        fs::write(path, text).expect("write a file");
    }

    fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(dir)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    /// `grund <args>` prints exactly `stdout`, writes nothing to stderr, and exits `code`.
    fn assert_prints(&self, args: &[&str], stdout: &str, code: i32) {
        let output = self.run_in(&self.0, args);
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

    /// The stdout lines of `grund <args>` that start with `prefix`, each with its newline.
    fn rows_at(&self, args: &[&str], prefix: &str) -> String {
        let output = self.run_in(&self.0, args);
        assert_eq!(output.status.code(), Some(1), "grund {args:?}: {output:?}");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.starts_with(prefix))
            .map(|line| format!("{line}\n"))
            .collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The controls, green today and after: a zero count names nothing and keeps its bytes and
/// `"sites":null`, the record that cites one item gets no row, and a check of a file whose
/// citations no count reads still succeeds (§FS-rules.7.3). They also prove the fixture sound,
/// so a red test below is about the list alone.
#[test]
fn a_zero_count_and_an_uncounted_file_keep_what_they_print_today() {
    let tree = Tree::new("controls");
    let zero = row("docs/labor/LABOR-002-mechanic.md", LABOR_002);
    assert_eq!(
        tree.rows_at(&["check"], "docs/labor/LABOR-002-mechanic.md:"),
        zero
    );
    assert_eq!(tree.rows_at(&["check"], &format!("{P3}:")), "");
    assert_eq!(
        tree.rows_at(
            &["check", "--format", "json"],
            "{\"severity\":\"error\",\"path\":\"docs/labor/LABOR-002"
        ),
        record(
            "uncited-unit",
            "docs/labor/LABOR-002-mechanic.md",
            LABOR_002,
            &[],
            "RULE-003-labor-used"
        )
    );
    tree.assert_prints(&["check", "docs/overview.md"], "success\n", 0);
}

/// The issue's whole run: every count above zero lists what it counted, the section citation
/// as written, the `cite each` row with its target ID still after `<subject> cites `, and the
/// inbound rows with citations from other files; the zero count is unchanged.
#[test]
fn every_count_above_zero_names_the_citations_it_counted() {
    let tree = Tree::new("text");
    let stdout = [
        row(
            "docs/inputs/INPUT-001-diesel.md",
            &format!("{INPUT_001}{}", clause(INPUT_COUNTED, "")),
        ),
        row(
            "docs/labor/LABOR-001-driver.md",
            &format!("{LABOR_001}{}", clause(LABOR_COUNTED, "")),
        ),
        row("docs/labor/LABOR-002-mechanic.md", LABOR_002),
        row(P1, &format!("{P1_ONE_ITEM}{}", clause(P1_COUNTED, ""))),
        row(P1, &format!("{P1_INPUT_ONCE}{}", clause(P1_COUNTED, ""))),
        row(P2, &format!("{P2_ONE_ITEM}{}", clause(P2_COUNTED, ""))),
    ]
    .concat();
    tree.assert_prints(&["check"], &stdout, 1);
    // The issue's own row, spelled out once so the clause's bytes are on the page.
    assert!(stdout.contains(
        "docs/prices/PRICE-001-diesel-retail.md:1: error: PRICE-001-diesel-retail cites INPUT or \
         LABOR 2 times; RULE-001-price-item requires exactly one; add or remove those citations \
         until the count fits; counted at docs/prices/PRICE-001-diesel-retail.md:7 \
         (INPUT-001-diesel), docs/prices/PRICE-001-diesel-retail.md:12 (INPUT-001-diesel.1)\n"
    ));
}

/// The same run in JSON: `message` carries the same clause, and `sites` each distinct counted
/// `{path, line}` in the clause's order; the zero count keeps `null` (§FS-rules.7.6,
/// §FS-errors.5.1).
#[test]
fn json_sites_carry_every_counted_line_in_text_order() {
    let tree = Tree::new("json");
    let stdout = [
        record(
            "uncited-unit",
            "docs/inputs/INPUT-001-diesel.md",
            &format!("{INPUT_001}{}", clause(INPUT_COUNTED, "")),
            &sites_of(INPUT_COUNTED),
            "RULE-002-input-use",
        ),
        record(
            "uncited-unit",
            "docs/labor/LABOR-001-driver.md",
            &format!("{LABOR_001}{}", clause(LABOR_COUNTED, "")),
            &sites_of(LABOR_COUNTED),
            "RULE-005-driver-once",
        ),
        record(
            "uncited-unit",
            "docs/labor/LABOR-002-mechanic.md",
            LABOR_002,
            &[],
            "RULE-003-labor-used",
        ),
        record(
            "citation-cardinality",
            P1,
            &format!("{P1_ONE_ITEM}{}", clause(P1_COUNTED, "")),
            &sites_of(P1_COUNTED),
            "RULE-001-price-item",
        ),
        record(
            "citation-cardinality",
            P1,
            &format!("{P1_INPUT_ONCE}{}", clause(P1_COUNTED, "")),
            &sites_of(P1_COUNTED),
            "RULE-004-input-once",
        ),
        record(
            "citation-cardinality",
            P2,
            &format!("{P2_ONE_ITEM}{}", clause(P2_COUNTED, "")),
            &sites_of(P2_COUNTED),
            "RULE-001-price-item",
        ),
    ]
    .concat();
    tree.assert_prints(&["check", "--format", "json"], &stdout, 1);
}

/// A `cite each` row reads only its one target's citations, so a second INPUT the record cites
/// is in the plain count's list and not in the `cite each` row's. Two citations on one line are
/// two entries in column order, not in the order of what they cite, and one site.
#[test]
fn a_cite_each_row_lists_only_its_own_targets_citations() {
    let tree = Tree::new("cite-each");
    tree.write(
        "docs/inputs/INPUT-002-gasoline.md",
        "# INPUT-002-gasoline: Gasoline\n\nRetail gasoline, per litre.\n",
    );
    tree.write(
        P1,
        &PRICE_001.replace(
            "from \u{a7}INPUT-001-diesel.1.",
            "against \u{a7}INPUT-002-gasoline and from \u{a7}INPUT-001-diesel.1.",
        ),
    );
    let three: Counted = &[
        (P1, 7, "INPUT-001-diesel"),
        (P1, 12, "INPUT-002-gasoline"),
        (P1, 12, "INPUT-001-diesel.1"),
    ];
    let one_item = P1_ONE_ITEM.replace("2 times", "3 times");
    assert_eq!(
        tree.rows_at(&["check"], &format!("{P1}:")),
        [
            row(P1, &format!("{one_item}{}", clause(three, ""))),
            row(P1, &format!("{P1_INPUT_ONCE}{}", clause(P1_COUNTED, ""))),
        ]
        .concat()
    );
    assert_eq!(
        tree.rows_at(
            &["check", "--format", "json"],
            &format!("{{\"severity\":\"error\",\"path\":\"{P1}\"")
        ),
        [
            record(
                "citation-cardinality",
                P1,
                &format!("{one_item}{}", clause(three, "")),
                &[(P1, 7), (P1, 12)],
                "RULE-001-price-item",
            ),
            record(
                "citation-cardinality",
                P1,
                &format!("{P1_INPUT_ONCE}{}", clause(P1_COUNTED, "")),
                &[(P1, 7), (P1, 12)],
                "RULE-004-input-once",
            ),
        ]
        .concat()
    );
}

/// A check of one citing file reports the inbound count its citation is part of, whole and at
/// the subject's title, and exits 1 (§FS-check.1.3.6,
/// §FS-check.requirements.narrow-never-lenient). Before #1366 it printed `success`.
#[test]
fn a_check_of_a_counted_citing_file_reports_the_count_it_is_part_of() {
    let tree = Tree::new("scoped-p3");
    tree.assert_prints(
        &["check", P3],
        &row(
            "docs/labor/LABOR-001-driver.md",
            &format!("{LABOR_001}{}", clause(LABOR_COUNTED, "")),
        ),
        1,
    );
}

/// A check of PRICE-002 reports its own row and both inbound rows one of its citations is
/// counted in; before #1366 it printed only its own.
#[test]
fn a_check_of_a_record_reports_every_count_its_citations_are_in() {
    let tree = Tree::new("scoped-p2");
    tree.assert_prints(
        &["check", P2],
        &[
            row(
                "docs/inputs/INPUT-001-diesel.md",
                &format!("{INPUT_001}{}", clause(INPUT_COUNTED, "")),
            ),
            row(
                "docs/labor/LABOR-001-driver.md",
                &format!("{LABOR_001}{}", clause(LABOR_COUNTED, "")),
            ),
            row(P2, &format!("{P2_ONE_ITEM}{}", clause(P2_COUNTED, ""))),
        ]
        .concat(),
        1,
    );
}

/// The report base spells the listed paths as it spells the row's own: under
/// `--path-base=invocation docs/prices` a counted citation inside is named from there, and a
/// subject outside, brought into scope by its counted sites, from `../` (§FS-errors.3).
#[test]
fn the_counted_list_is_spelled_from_the_report_base() {
    let tree = Tree::new("path-base");
    let base = "docs/prices";
    tree.assert_prints(
        &["check", "--path-base=invocation", base],
        &[
            row(
                "../inputs/INPUT-001-diesel.md",
                &format!("{INPUT_001}{}", clause(INPUT_COUNTED, base)),
            ),
            row(
                "../labor/LABOR-001-driver.md",
                &format!("{LABOR_001}{}", clause(LABOR_COUNTED, base)),
            ),
            row(
                "PRICE-001-diesel-retail.md",
                &format!("{P1_ONE_ITEM}{}", clause(P1_COUNTED, base)),
            ),
            row(
                "PRICE-001-diesel-retail.md",
                &format!("{P1_INPUT_ONCE}{}", clause(P1_COUNTED, base)),
            ),
            row(
                "PRICE-002-haul.md",
                &format!("{P2_ONE_ITEM}{}", clause(P2_COUNTED, base)),
            ),
        ]
        .concat(),
        1,
    );
}

/// On a file with two names the identity clause stays directly after the remedy, so the released
/// two-name row stays a prefix, and the counted list follows it; `sites` holds the row's own
/// spellings and each counted line's, as the identity pass grows every finding's sites
/// (§FS-rules.7.3, §FS-scan.identity.findings.content).
#[cfg(unix)]
#[test]
fn on_a_file_with_two_names_the_list_follows_the_identity_clause() {
    let tree = Tree::new("two-names");
    fs::create_dir_all(tree.0.join("docs/more")).expect("create docs/more");
    std::os::unix::fs::symlink(
        "../prices/PRICE-002-haul.md",
        tree.0.join("docs/more/haul-link.md"),
    )
    .expect("link the haul record");
    let link = "docs/more/haul-link.md";
    let message = format!("{P2_ONE_ITEM} (also at {link}:1){}", clause(P2_COUNTED, ""));
    assert_eq!(
        tree.rows_at(&["check"], &format!("{P2}:")),
        row(P2, &message)
    );
    assert!(message.contains(
        "until the count fits (also at docs/more/haul-link.md:1); counted at \
         docs/prices/PRICE-002-haul.md:5 (INPUT-001-diesel), docs/prices/PRICE-002-haul.md:8 \
         (LABOR-001-driver)"
    ));
    assert_eq!(
        tree.rows_at(
            &["check", "--format", "json"],
            &format!("{{\"severity\":\"error\",\"path\":\"{P2}\"")
        ),
        record(
            "citation-cardinality",
            P2,
            &message,
            &[(P2, 1), (link, 1), (P2, 5), (link, 5), (P2, 8), (link, 8)],
            "RULE-001-price-item",
        )
    );
}
