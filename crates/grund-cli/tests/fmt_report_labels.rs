//! The dry-run report's labels are the ones the specification lists
//! (§FS-fmt.3.5). Every label `grund fmt --check` prints is collected from a
//! fixture that fires each rewrite once, then looked up in the spec points that
//! name them, so a label the binary ships and no spec point declares fails here
//! rather than in a reader quoting the report (agent-grounds/grund#408).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One project under `[reference] shorthand = "canonical"`, with
/// `FS-042-user-login` to expand to and `FS-004-quick-actions` carrying `body`.
fn fixture(name: &str, body: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("grund-fmt-labels-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/functional-spec")).expect("create fixture");
    fs::write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"fmt-labels\"\n\n",
            "[reference]\nmarker = \"\u{a7}\"\nstrict = true\nshorthand = \"canonical\"\n\n",
            "[id]\nformat = \"{kind}-{number}-{slug}\"\nsection_separator = \".\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\ntitle = \"What\"\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
        ),
    )
    .expect("write config");
    fs::write(
        root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: Login\n\nLead.\n\n## 1. One\n\nText.\n",
    )
    .expect("write target");
    fs::write(
        root.join("docs/functional-spec/FS-004-quick-actions.md"),
        body,
    )
    .expect("write citing declaration");
    root
}

/// Each line fires exactly one rewrite: a typed trigger, a bare ID (with
/// `--marker`), a number-only shorthand (§FS-fmt.2.4), a declaration-local token
/// (§FS-fmt.2.4, second paragraph), and a marked citation for `--cross-refs`.
const ONE_REWRITE_PER_LINE: &str = concat!(
    "# FS-004-quick-actions: Quick actions\n\n",
    "Typed: $$FS-042-user-login here.\n\n",
    "Bare: FS-042-user-login here.\n\n",
    "A number-only shorthand: \u{a7}FS-042 is one.\n\n",
    "A declaration-local token: \u{a7}1 is the other.\n\n",
    "Marked: \u{a7}FS-042-user-login already.\n\n",
    "## 1. One\n",
);

/// Lines that each fire more than one rewrite (§FS-fmt.3.5.1, §FS-fmt.3.5.2).
const SEVERAL_REWRITES_PER_LINE: &str = concat!(
    "# FS-004-quick-actions: Quick actions\n\n",
    "Typed and expanded: $$FS-042 here.\n\n",
    "Expanded and wrapped: \u{a7}FS-042 here.\n\n",
    "Two local tokens: \u{a7}1 and \u{a7}2 here.\n\n",
    "Typed beside a local token: $$FS-042-user-login and \u{a7}1 here.\n\n",
    "Withheld: \u{a7}9 and \u{a7}1 here.\n\n",
    "Bare beside a shorthand: FS-042-user-login and \u{a7}FS-042 here.\n\n",
    "## 1. One\n\n",
    "## 2. Two\n",
);

/// The `grund fmt --check <flag>` report rows, with the citing file's path cut.
fn report(root: &Path, flag: &str) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["fmt", "--check", flag])
        .current_dir(root)
        .output()
        .expect("run grund fmt --check");
    assert_eq!(
        output.status.code(),
        Some(1),
        "fmt --check {flag} found nothing to rewrite"
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 stdout")
        .lines()
        .map(|row| {
            row.strip_prefix("docs/functional-spec/FS-004-quick-actions.md:")
                .unwrap_or_else(|| panic!("unexpected report row {row:?}"))
                .to_string()
        })
        .collect()
}

/// `path:line: <kind>[: <from> → <to>]` → `(line, kind)` (§FS-fmt.3.5, §FS-fmt.3.6).
fn reported_labels(root: &Path) -> Vec<(usize, String)> {
    let mut labels = Vec::new();
    for flag in ["--marker", "--cross-refs"] {
        for rest in report(root, flag) {
            let (line, kind) = rest.split_once(": ").expect("path:line: <kind>");
            let kind = kind.split(": ").next().expect("kind").to_string();
            labels.push((line.parse().expect("line number"), kind));
        }
    }
    labels
}

/// The text of one numbered heading's body in `FS-fmt.md`, cut at the next
/// heading of `stop` depth or shallower.
fn spec_section(heading: &str, stop: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/functional-spec/FS-fmt.md");
    let spec = fs::read_to_string(path).expect("read FS-fmt.md");
    let start = spec
        .find(&format!("\n{heading}"))
        .unwrap_or_else(|| panic!("FS-fmt.md has no {heading:?}"));
    let body = &spec[start + 1 + heading.len()..];
    let end = body
        .match_indices(&format!("\n{stop}"))
        .map(|(at, _)| at)
        .next()
        .unwrap_or(body.len());
    body[..end].to_string()
}

#[test]
fn every_dry_run_label_the_binary_prints_is_named_by_the_dry_run_report_point() {
    let root = fixture("report", ONE_REWRITE_PER_LINE);
    let printed = reported_labels(&root)
        .into_iter()
        .map(|(_, kind)| kind)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        printed.len(),
        5,
        "the fixture should fire all five rewrites: {printed:?}"
    );
    let report = spec_section("### 3.5 The dry-run report", "### ");
    let unnamed = printed
        .iter()
        .filter(|kind| !report.contains(&format!("`{kind}`")))
        .collect::<Vec<_>>();
    assert!(
        unnamed.is_empty(),
        "fmt --check prints {unnamed:?}, which \u{a7}FS-fmt.3.5 does not name"
    );
}

#[test]
fn the_shorthand_point_names_the_label_each_of_its_expansions_is_reported_under() {
    let root = fixture("shorthand", ONE_REWRITE_PER_LINE);
    let labels = reported_labels(&root);
    let label_at = |line: usize| {
        labels
            .iter()
            .find(|(at, _)| *at == line)
            .map(|(_, kind)| kind.clone())
            .unwrap_or_else(|| panic!("no report row for line {line}"))
    };
    let (number_only, local) = (label_at(7), label_at(9));
    let lead = spec_section("### 2.4 Shorthand-to-canonical", "#### ");
    for kind in [&number_only, &local] {
        assert!(
            lead.contains(&format!("`{kind}`")),
            "\u{a7}FS-fmt.2.4 does not name `{kind}`, \
             the label one of its expansions is reported under"
        );
    }
    assert!(
        !lead.contains("labels each expansion `shorthand \u{2192} canonical`"),
        "\u{a7}FS-fmt.2.4 claims every expansion is labelled `shorthand \u{2192} canonical`, \
         but the declaration-local one is reported as `{local}`"
    );
    let named_text = spec_section(
        "### 3.6 An expanded shorthand names the text it writes",
        "### ",
    );
    assert!(
        named_text.contains(&format!("`{local}")),
        "\u{a7}FS-fmt.3.6 does not say the `{local}` row names the text it writes"
    );
}

#[test]
fn a_line_with_several_rewrites_is_one_row_under_the_first_label() {
    let root = fixture("several", SEVERAL_REWRITES_PER_LINE);
    assert_eq!(
        report(&root, "--cross-refs"),
        [
            // §FS-fmt.3.5.1: marked then expanded reads as the trigger, expanded
            // then wrapped as the shorthand; both still name what they write.
            "3: trigger \u{2192} marker: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login",
            "5: shorthand \u{2192} canonical: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login",
            // §FS-fmt.3.5.2: one row however many local tokens, under the local label
            // unless an earlier rewrite fired; a withheld token (§FS-fmt.2.4.6) has no detail.
            "7: local section \u{2192} canonical: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1, \u{a7}2 \u{2192} \u{a7}FS-004-quick-actions.2",
            "9: trigger \u{2192} marker: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1",
            "11: local section \u{2192} canonical: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1",
            // §FS-fmt.3.5.1: wrapping never outranks an expansion.
            "13: shorthand \u{2192} canonical: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login",
        ]
    );
    // §FS-fmt.3.6: a bare citation marked on a line that expanded a shorthand
    // still names what the expansion writes.
    let marked = report(&root, "--marker");
    assert!(
        marked.contains(
            &"13: bare \u{2192} marker: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login".to_string()
        ),
        "fmt --check --marker rows: {marked:?}"
    );
}
