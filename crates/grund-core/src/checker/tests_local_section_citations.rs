//! Independent form, ownership, syntax, and target-existence findings for local
//! numeric section citations (§FS-check.3.2, §FS-check.3.24, §AR-checker), the
//! release pair every one of them names (§FS-check.3.24.1, §FS-check.3.24.2), and
//! the escape an owned site is offered where its owner lacks the section
//! (§FS-check.3.24.3).

use super::references::{LOCAL_SECTION_RULE_PRIOR_RELEASE, LOCAL_SECTION_RULE_RELEASE};
use super::*;
use crate::api::CheckRun;
use crate::config::{display_path, load_config};
use crate::testing::{
    check_run, located_diagnostics, numbered_config, scan_tree, test_root, write,
};

#[test]
fn local_section_findings_are_actionable_and_missing_is_independent() {
    let root = test_root("local_section_findings_are_actionable_and_missing_is_independent");
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        concat!(
            "# FS-001-alpha: Alpha\n\n",
            "Valid \u{a7}2.\n",
            "Missing \u{a7}9.9.\n",
            "Unsupported \u{a7}2.goals and \u{a7}2abc.\n",
            "Malformed \u{a7}2..1 and \u{a7}2... and \u{a7}2..goals.\n\n",
            "## 2. Target\n",
        ),
    );
    write(
        &root.join("docs/notes.md"),
        "# Notes\n\nOwnerless \u{a7}2.\n",
    );
    let config = numbered_config(root.clone());
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan fixture");
    assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
    let report = check_findings(&findings, &config);
    let mut actual = report
        .errors
        .iter()
        .map(|finding| (finding.code, finding.message.as_str()))
        .collect::<Vec<_>>();
    actual.sort_unstable_by_key(|(_, message)| *message);
    let mut expected = vec![
        (
            "local-section-citation",
            "local section citation \u{a7}2; write \u{a7}FS-001-alpha.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
        ),
        (
            "local-section-citation",
            // §FS-check.3.24.3: `FS-001-alpha` has no section 9.9, so the finding
            // says so and offers the escape, and the command goes with the
            // rewrite §FS-fmt.2.4.6 refuses.
            "local section citation \u{a7}9.9; FS-001-alpha has no section 9.9, so write a full citation or <§>9.9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        ("missing-section", "missing section FS-001-alpha.9.9"),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2.goals; write a full citation or <§>2.goals to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2abc; write a full citation or <§>2abc to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2..1; write a full citation or <§>2..1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2...; write a full citation or <§>2... to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2..goals; write a full citation or <§>2..goals to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
        (
            "local-section-citation",
            "local section citation \u{a7}2 has no enclosing declaration; write a full citation or <§>2 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ),
    ];
    expected.sort_unstable_by_key(|(_, message)| *message);

    assert_eq!(actual, expected);
    assert!(
        !report
            .warnings
            .iter()
            .any(|finding| finding.code == "unused"),
        "owned local edges count as uses: {:?}",
        report
            .warnings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn owned_local_edges_feed_grounding_unused_and_citation_direction_checks() {
    let root = test_root("owned_local_edges_feed_grounding_unused_and_citation_direction_checks");
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "[reference]\nstrict = true\nrequire_grounding = true\n",
            "[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
            "[citations]\ndefault = \"may\"\n",
            "[citations.FS]\nmust-not = [\"FS\"]\n",
        ),
    );
    write(
        &root.join("docs/FS-alpha.md"),
        "# FS-alpha: Alpha\n\nLocal \u{a7}2.\n\n## 2. Target\n",
    );
    let config = load_config(&root).expect("load direction config");
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan fixture");
    assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
    let report = check_findings(&findings, &config);
    let messages = report
        .errors
        .iter()
        .chain(&report.warnings)
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("must not cite FS")),
        "the local edge reaches citation directions: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("ungrounded")
                && !message.contains("declared but never cited")),
        "the same edge grounds and counts as use: {messages:?}"
    );
}

/// Ordering only, so the `-dev` suffix is dropped rather than modelled — the
/// same reading `index_rule_releases_are_ordered_and_behind_us` takes of the
/// pair it holds.
fn version(text: &str) -> (u64, u64, u64) {
    let mut parts = text.split('.').map(|part| {
        part.split(|ch: char| !ch.is_ascii_digit())
            .next()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("not a version: {text}"))
    });
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// §FS-check.3.24.1: both halves of the pair are claims about releases that
/// happened — a verdict this rule *moved between* — written in the past-tense
/// half of the vocabulary §FS-distribution.4.2 closes, so a tree that printed
/// either as a version still ahead would be promising rather than reporting.
/// The same shape `index_rule_releases_are_ordered_and_behind_us` holds the
/// neighbouring rule's pair to.
#[test]
fn local_section_rule_releases_are_ordered_and_behind_us() {
    let current = version(env!("CARGO_PKG_VERSION"));
    let prior = version(LOCAL_SECTION_RULE_PRIOR_RELEASE);
    let arrival = version(LOCAL_SECTION_RULE_RELEASE);
    assert!(
        prior < arrival,
        "{LOCAL_SECTION_RULE_PRIOR_RELEASE} < {LOCAL_SECTION_RULE_RELEASE}"
    );
    assert!(
        prior <= current,
        "§FS-check.3.24.1 says the form was unchecked in {LOCAL_SECTION_RULE_PRIOR_RELEASE}, which has to be a release that happened (this tree is {})",
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        arrival <= current,
        "§FS-check.3.24.1 says the verdict moved in {LOCAL_SECTION_RULE_RELEASE}, which has to be a release that happened rather than one still ahead (this tree is {})",
        env!("CARGO_PKG_VERSION")
    );
}

/// §FS-check.3.24.1's third withholding case: out past `[scan] include`, `fmt`
/// does not reach the site either, so the command offer goes for the reason
/// §FS-check.3.14.4 withholds the sibling's — while the tier still leads the
/// message and the attribution still trails it (§FS-check.3.14.6). The fixture
/// writes the *same* owned construct twice, once under `include` and once past
/// it, so the one difference the tier makes is the whole of the diff.
#[test]
fn the_out_of_scope_tier_withholds_the_command_but_keeps_the_attribution() {
    let root = test_root("the_out_of_scope_tier_withholds_the_command_but_keeps_the_attribution");
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/in\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs/in\"]\nextensions = [\"md\"]\n",
        ),
    );
    write(
        &root.join("docs/in/FS-in.md"),
        "# FS-in: Under include\n\nOwned \u{a7}2.\n\n## 2. Target\n",
    );
    // Out past `include` *and* past the kind home, because §FS-config.3.5.8 makes
    // every home a configured-scope root: a second file under `docs` would be in
    // scope however narrow `include` is.
    write(
        &root.join("sim/FS-out.md"),
        "# FS-out: Past include\n\nOwned \u{a7}2.\n\n## 2. Target\n",
    );

    let full = check_run(&root, true);
    let local = full
        .report
        .errors
        .iter()
        .filter(|error| error.code.ends_with("local-section-citation"))
        .collect::<Vec<_>>();
    assert_eq!(
        located_diagnostics(&full.config, local),
        vec![
            "docs/in/FS-in.md:3: local section citation \u{a7}2; write \u{a7}FS-in.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
            "sim/FS-out.md:3: outside [scan] include: local section citation \u{a7}2; write \u{a7}FS-out.2 — unchecked in grund 0.13.1, an error in 0.14.0",
        ],
        "§FS-check.3.24.1: the same site keeps the attribution and loses only the command once it is out of the configured scope"
    );
}

/// §FS-check.3.24.1's fourth withholding case, over all three shapes
/// §FS-fmt.2.4.6 refuses: a wholly absent path, a partially resolving one, and
/// an owner with no numbered sections at all. A resolving control sits in the
/// same tree under the same tier, so the one difference the predicate makes is
/// the whole of the diff. Each refused site says on its own line that the owner
/// lacks the section and offers the escape (§FS-check.3.24.3), and the
/// `missing section` error still stays beside it (§FS-check.3.2).
#[test]
fn an_absent_target_section_withholds_the_command_and_keeps_both_findings() {
    let root = test_root("an_absent_target_section_withholds_the_command_and_keeps_both_findings");
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
        ),
    );
    write(
        &root.join("docs/FS-nine.md"),
        concat!(
            "# FS-nine: The owner stops short\n\n",
            "Resolving control \u{a7}2.1.\n",
            "Wholly absent \u{a7}12.\n",
            "Partly resolving \u{a7}2.7.\n\n",
            "## 2. Target\n\n### 2.1 Child\n",
        ),
    );
    write(
        &root.join("docs/FS-bare.md"),
        "# FS-bare: No numbered section at all\n\nNone to resolve against \u{a7}1.\n",
    );

    let run = check_run(&root, false);
    let local = run
        .report
        .errors
        .iter()
        .filter(|error| error.code == "local-section-citation" || error.code == "missing-section")
        .collect::<Vec<_>>();
    assert_eq!(
        located_diagnostics(&run.config, local),
        vec![
            "docs/FS-bare.md:3: local section citation \u{a7}1; FS-bare has no section 1, so write a full citation or <§>1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-bare.md:3: missing section FS-bare.1",
            "docs/FS-nine.md:3: local section citation \u{a7}2.1; write \u{a7}FS-nine.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
            "docs/FS-nine.md:4: local section citation \u{a7}12; FS-nine has no section 12, so write a full citation or <§>12 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:4: missing section FS-nine.12",
            "docs/FS-nine.md:5: local section citation \u{a7}2.7; FS-nine has no section 2.7, so write a full citation or <§>2.7 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:5: missing section FS-nine.2.7",
        ],
        "§FS-fmt.2.4.6: only the control keeps the command clause, and every refused site keeps its pair"
    );
}

/// The config every §FS-check.3.24.3 fixture below starts from: one `FS` home
/// under `docs`, and `extra` appended for the one key a case is about.
fn absent_target_config(extra: &str) -> String {
    format!(
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{{kind}}-{{slug}}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n{}",
        ),
        extra
    )
}

/// §FS-check.3.24.3's condition, read off one run: an owned site is answered with
/// the escape exactly where `missing section` names the same coordinate on the
/// same line, and with the full citation exactly where it does not. Returns the
/// sites that broke it, so the assertion names them.
fn escape_follows_missing_section(run: &CheckRun) -> Vec<String> {
    let marker = run.config.marker.as_str();
    let mut broken = Vec::new();
    for finding in &run.report.errors {
        let Some(rest) = finding
            .message
            .strip_prefix("local section citation ")
            .filter(|_| finding.code == "local-section-citation")
        else {
            continue;
        };
        let Some((token, advice)) = rest.split_once("; ") else {
            continue; // the ownerless shape, which has no owner to lack anything
        };
        let path = token.strip_prefix(marker).unwrap_or(token);
        let missing = run.report.errors.iter().any(|other| {
            other.code == "missing-section"
                && other.path == finding.path
                && other.line == finding.line
                && other
                    .message
                    .ends_with(&format!("{}{path}", run.config.section_separator))
        });
        let escaped = advice.contains(&format!(
            " has no section {path}, so write a full citation or <{marker}>{path} to show the shape without citing it"
        ));
        let written = advice.starts_with(&format!("write {marker}"));
        if missing != escaped || missing == written {
            broken.push(format!(
                "{}:{}: missing section beside it: {missing}; {:?}",
                display_path(&run.config, finding.path.as_ref().expect("located")),
                finding.line.unwrap_or(0),
                finding.message
            ));
        }
    }
    broken
}

/// §FS-check.3.24.3 over every place an owned site can sit: the wording turns on
/// the owner alone, so a resolving control, a partly resolving path, a wholly
/// absent one, an owner with no numbered sections, a protected inline-code site,
/// and a `[fmt] exclude` file each read as the condition says, and only the
/// resolving sites the formatter would write keep the command clause
/// (§FS-check.3.24.1). The `missing section` error stays beside every absent site
/// (§FS-check.3.2), unchanged.
#[test]
fn an_absent_target_section_is_answered_with_the_escape_exactly_where_missing_section_fires() {
    let root = test_root(
        "an_absent_target_section_is_answered_with_the_escape_exactly_where_missing_section_fires",
    );
    write(
        &root.join("grund.toml"),
        &absent_target_config("\n[fmt]\nexclude = [\"docs/FS-excluded.md\"]\n"),
    );
    write(
        &root.join("docs/FS-nine.md"),
        concat!(
            "# FS-nine: The owner stops short\n\n",
            "Resolving control \u{a7}2.1.\n",
            "Wholly absent \u{a7}12.\n",
            "Partly resolving \u{a7}2.7.\n",
            "In inline code `\u{a7}9` and `\u{a7}2`.\n\n",
            "## 2. Target\n\n### 2.1 Child\n",
        ),
    );
    write(
        &root.join("docs/FS-bare.md"),
        "# FS-bare: No numbered section at all\n\nNone to resolve against \u{a7}1.\n",
    );
    write(
        &root.join("docs/FS-excluded.md"),
        "# FS-excluded: Out of the formatter's reach\n\nResolving \u{a7}2 and absent \u{a7}3.\n\n## 2. Target\n",
    );

    let run = check_run(&root, false);
    let local = run
        .report
        .errors
        .iter()
        .filter(|error| error.code == "local-section-citation" || error.code == "missing-section")
        .collect::<Vec<_>>();
    assert_eq!(
        located_diagnostics(&run.config, local),
        vec![
            "docs/FS-bare.md:3: local section citation \u{a7}1; FS-bare has no section 1, so write a full citation or <§>1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-bare.md:3: missing section FS-bare.1",
            "docs/FS-excluded.md:3: local section citation \u{a7}2; write \u{a7}FS-excluded.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
            "docs/FS-excluded.md:3: local section citation \u{a7}3; FS-excluded has no section 3, so write a full citation or <§>3 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-excluded.md:3: missing section FS-excluded.3",
            "docs/FS-nine.md:3: local section citation \u{a7}2.1; write \u{a7}FS-nine.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
            "docs/FS-nine.md:4: local section citation \u{a7}12; FS-nine has no section 12, so write a full citation or <§>12 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:4: missing section FS-nine.12",
            "docs/FS-nine.md:5: local section citation \u{a7}2.7; FS-nine has no section 2.7, so write a full citation or <§>2.7 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:5: missing section FS-nine.2.7",
            "docs/FS-nine.md:6: local section citation \u{a7}2; write \u{a7}FS-nine.2 — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:6: local section citation \u{a7}9; FS-nine has no section 9, so write a full citation or <§>9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-nine.md:6: missing section FS-nine.9",
        ],
        "§FS-check.3.24.3: every absent site names the absence and the escape; every resolving one keeps today's bytes"
    );
    assert_eq!(
        escape_follows_missing_section(&run),
        Vec::<String>::new(),
        "§FS-check.3.24.3: the escape appears exactly where `missing section` names the same coordinate"
    );
}

/// §FS-check.3.24.3's words under a configured marker and a custom separator: the
/// escape carries the marker, and the path the finding says the owner lacks is the
/// token's own dotted path (§FS-check.1.1.8), so the separator `missing section`
/// puts between the owner and the path appears in neither. The resolving control
/// keeps both, as today.
#[test]
fn the_escape_carries_the_configured_marker_and_the_path_without_the_separator() {
    let root =
        test_root("the_escape_carries_the_configured_marker_and_the_path_without_the_separator");
    write(
        &root.join("grund.toml"),
        &absent_target_config("")
            .replace("strict = true", "marker = \"@\"\nstrict = true")
            .replace(
                "format = \"{kind}-{slug}\"",
                "format = \"{kind}-{slug}\"\nsection_separator = \":\"",
            ),
    );
    write(
        &root.join("docs/FS-a.md"),
        "# FS-a: A thing\n\nResolving @2.\nAbsent @5.1.\n\n## 2. Target\n",
    );

    let run = check_run(&root, false);
    assert_eq!(run.config.marker, "@", "the fixture configures the marker");
    assert_eq!(run.config.section_separator, ":", "and the separator");
    let local = run
        .report
        .errors
        .iter()
        .filter(|error| error.code == "local-section-citation" || error.code == "missing-section")
        .collect::<Vec<_>>();
    assert_eq!(
        located_diagnostics(&run.config, local),
        vec![
            "docs/FS-a.md:3: local section citation @2; write @FS-a:2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
            "docs/FS-a.md:4: local section citation @5.1; FS-a has no section 5.1, so write a full citation or <@>5.1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "docs/FS-a.md:4: missing section FS-a:5.1",
        ],
        "§FS-check.3.24.3: `<@>5.1`, and `5.1` rather than `:5.1`"
    );
    assert_eq!(escape_follows_missing_section(&run), Vec::<String>::new());
}

/// §FS-check.3.24.3 out past `[scan] include` under `--full`: the same absent site
/// is answered with the same escape, the scope still leads the message
/// (§FS-check.3.14.6), and the attribution still trails it with no command after
/// it. The in-scope twin shows the tier changes nothing but the lead.
#[test]
fn an_out_of_scope_absent_site_keeps_the_scope_first_and_answers_with_the_escape() {
    let root =
        test_root("an_out_of_scope_absent_site_keeps_the_scope_first_and_answers_with_the_escape");
    write(
        &root.join("grund.toml"),
        &absent_target_config("")
            .replace("folder = \"docs\"", "folder = \"docs/in\"")
            .replace("include = [\"docs\"]", "include = [\"docs/in\"]"),
    );
    write(
        &root.join("docs/in/FS-in.md"),
        "# FS-in: Under include\n\nOwned \u{a7}9.\n\n## 2. Target\n",
    );
    // Past `include` and past the kind home, for the reason the tier test above gives.
    write(
        &root.join("sim/FS-out.md"),
        "# FS-out: Past include\n\nOwned \u{a7}9.\n\n## 2. Target\n",
    );

    let full = check_run(&root, true);
    let local = full
        .report
        .errors
        .iter()
        .filter(|error| error.code.ends_with("local-section-citation"))
        .collect::<Vec<_>>();
    assert_eq!(
        located_diagnostics(&full.config, local),
        vec![
            "docs/in/FS-in.md:3: local section citation \u{a7}9; FS-in has no section 9, so write a full citation or <§>9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
            "sim/FS-out.md:3: outside [scan] include: local section citation \u{a7}9; FS-out has no section 9, so write a full citation or <§>9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0",
        ],
        "§FS-check.3.14.6: the scope leads; §FS-check.3.24.3: the escape follows"
    );
}
