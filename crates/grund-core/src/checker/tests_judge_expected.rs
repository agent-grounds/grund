//! Test module: the two halves and what they are handed (§AR-checker.1). `judge`
//! compares the `Expected` bytes the writers rendered and renders nothing
//! (§AR-checker.1.3), the managed-block and index-target reads go through those
//! bytes (§AR-checker.2.7, §AR-checker.2.16), and the merge keeps the single
//! driver's order (§AR-checker.1.4).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::conform::conform;
use super::judge::{CheckWorkspace, judge};
use super::report::check_on_disk;
use super::support::sort_diagnostics;
use crate::config::{Config, load_config};
use crate::model::{Catalog, CheckReport, Diagnostic, Expected, TextOverlays};
use crate::testing::{
    drifted_include_repo, kind_index_repo, linked_repo, scan_tree, test_root, write,
};
use crate::writers::expected;

type Line = (&'static str, Option<PathBuf>, Option<usize>, String);

fn lines<'a>(diagnostics: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Line> {
    diagnostics
        .into_iter()
        .map(|d| (d.code, d.path.clone(), d.line, d.message.clone()))
        .collect()
}

/// Every finding of `report` but the managed-block drift, per channel.
fn all_but_drift(report: &CheckReport) -> [Vec<Line>; 3] {
    let kept = |channel: &[Diagnostic]| lines(channel.iter().filter(|d| d.code != "agents-init"));
    [
        kept(&report.errors),
        kept(&report.warnings),
        kept(&report.suggestions),
    ]
}

/// The file names the managed-block drift findings sit in, in report order.
fn drifted(report: &CheckReport) -> Vec<String> {
    report
        .errors
        .iter()
        .filter(|d| d.code == "agents-init")
        .filter_map(|d| d.path.as_deref()?.file_name()?.to_str().map(str::to_string))
        .collect()
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create fixture directory");
    for entry in fs::read_dir(from).expect("read fixture directory") {
        let entry = entry.expect("fixture entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("fixture entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

/// The e2e baseline of §FS-check.3.5.4 — a chapter rule, an `AGENTS.md` and a
/// non-symlink `CLAUDE.md` whose managed blocks `init` wrote under
/// `anchor_format = "github"`, a dangling citation and two rule violations — copied
/// so a case may edit it. `None` from a packaged crate, which carries no e2e tree.
fn presentation_fixture(name: &str) -> Option<PathBuf> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-presentation-only-drift-baseline/repo");
    if !source.is_dir() {
        return None;
    }
    let root = test_root(name);
    copy_tree(&source, &root);
    Some(root)
}

fn loaded(root: &Path) -> (Config, Catalog) {
    let config = load_config(root).expect("load config");
    let (catalog, _) = scan_tree(&config, Some(root), true).expect("scan");
    (config, catalog)
}

fn judged(config: &Config, catalog: &Catalog, expected: &Expected) -> CheckReport {
    let none = BTreeMap::new();
    let workspace = CheckWorkspace::alone(&none);
    judge(
        config.rules(),
        config.schema(),
        catalog,
        expected,
        config.frame(),
        &workspace,
    )
}

/// §AR-checker.1.2, §AR-checker.1.3: with schema, rules and catalog fixed, two
/// presentations reach `judge` only as two `Expected`s, and the managed-block
/// drift is the one verdict they can move.
#[test]
fn judge_under_two_presentations_differs_only_in_managed_block_drift() {
    let Some(root) = presentation_fixture("judge_under_two_presentations") else {
        return;
    };
    let (github, catalog) = loaded(&root);
    let mut pandoc = github.clone();
    pandoc
        .edit_project(|project| project.presentation.fmt.anchor_format = "pandoc".to_string())
        .expect("flip the anchor profile");
    assert_eq!(github.schema(), pandoc.schema(), "the schema is fixed");
    assert_eq!(github.rules(), pandoc.rules(), "the rules are fixed");

    let none = BTreeMap::new();
    let as_written = judged(&github, &catalog, &expected(&catalog, &github, &none));
    let flipped = judged(&github, &catalog, &expected(&catalog, &pandoc, &none));

    assert_eq!(
        drifted(&as_written),
        Vec::<String>::new(),
        "the blocks are current"
    );
    assert_eq!(
        drifted(&flipped),
        ["AGENTS.md", "CLAUDE.md"],
        "one drift per entrypoint"
    );
    assert_eq!(
        all_but_drift(&as_written),
        all_but_drift(&flipped),
        "no other verdict moves"
    );
    assert!(
        !all_but_drift(&flipped)[0].is_empty(),
        "the fixture's own findings are judged under both"
    );
}

/// §AR-checker.2.7: the root block and its companion are each compared against
/// the bytes rendered for that file, so a companion that drifted alone is the
/// one finding; and a companion probe that failed is the `io` finding it was.
#[test]
fn each_entrypoint_is_compared_against_its_own_bytes_and_a_probe_error_is_an_io_finding() {
    let Some(root) = presentation_fixture("each_entrypoint_its_own_bytes") else {
        return;
    };
    let (config, catalog) = loaded(&root);
    let rendered = expected(&catalog, &config, &BTreeMap::new());
    let paths = rendered
        .entrypoints
        .iter()
        .map(|entrypoint| (entrypoint.path.clone(), entrypoint.require_block))
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            (root.join("AGENTS.md"), true),
            (root.join("CLAUDE.md"), true)
        ],
        "the root block first, then its companion"
    );

    let companion = root.join("CLAUDE.md");
    let text = fs::read_to_string(&companion).expect("read companion");
    let tampered = text.replacen(
        "- Each FS must have exactly one Terms chapter.",
        "- Each FS should have exactly one Terms chapter.",
        1,
    );
    assert_ne!(tampered, text, "the fixture renders the rule's bullet");
    write(&companion, &tampered);
    let report = judged(&config, &catalog, &rendered);
    assert_eq!(
        drifted(&report),
        ["CLAUDE.md"],
        "only the companion drifted"
    );

    let probe = Expected {
        entrypoint_probe_error: Some((companion.clone(), "CLAUDE.md: denied".to_string())),
        ..Expected::default()
    };
    let report = judged(&config, &catalog, &probe);
    assert_eq!(
        lines(report.errors.iter().filter(|d| d.code == "io")),
        [(
            "io",
            Some(companion),
            Some(1),
            "CLAUDE.md: denied".to_string()
        )],
    );
}

/// An `AR` kind whose index enrolls the inline declaration in `src/bus.rs` by its
/// canonical bare-ID source link (§FS-check.3.18.3).
fn external_index_repo(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
             [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n\n\
             [scan]\ninclude = [\"docs\", \"src\"]\n",
    );
    write(
        &root.join("src/bus.rs"),
        "/// AR-001-bus: The in-process event bus\n///\n/// Broadcasts in order.\nfn bus() {}\n",
    );
    write(
        &root.join("docs/architecture/README.md"),
        "# Architecture\n\n- [§AR-001-bus](../../src/bus.rs)\n",
    );
    root
}

fn has_code(report: &CheckReport, code: &str) -> bool {
    [&report.errors, &report.warnings, &report.suggestions]
        .into_iter()
        .flatten()
        .any(|d| d.code == code)
}

/// §AR-checker.2.16: whether the link enrolls is a lookup of `Expected`'s index
/// target. Under the rendered target the link is navigation, so the declaration
/// is unused; under any other it is an ordinary citation, and so a use.
#[test]
fn an_external_enrollment_is_matched_by_the_expected_index_target() {
    let root = external_index_repo("external_enrollment_by_expected");
    let (config, catalog) = loaded(&root);
    let rendered = expected(&catalog, &config, &BTreeMap::new());
    assert!(
        !rendered.index_targets.is_empty(),
        "the index citation has a target"
    );

    let enrolled = judged(&config, &catalog, &rendered);
    assert!(!has_code(&enrolled, "missing-index-entry"));
    assert!(has_code(&enrolled, "unused"), "the enrollment is not a use");

    let mut elsewhere = rendered.clone();
    for target in elsewhere.index_targets.values_mut() {
        *target = "../../src/elsewhere.rs".to_string();
    }
    let not_enrolled = judged(&config, &catalog, &elsewhere);
    assert!(
        !has_code(&not_enrolled, "unused"),
        "a link that is not the expected target is an ordinary citation"
    );
}

/// §AR-checker.2.16, §AR-resolver.6: a `.rs` home's link carries no heading
/// anchor, so the anchor profile presentation picks leaves its target alone, and
/// the declaration enrolls under both.
#[test]
fn an_inline_source_declaration_enrolls_under_both_anchor_profiles() {
    let root = external_index_repo("source_enrolls_under_both_profiles");
    let (github, catalog) = loaded(&root);
    let mut targets = Vec::new();
    for profile in ["github", "pandoc"] {
        let mut config = github.clone();
        config
            .edit_project(|project| project.presentation.fmt.anchor_format = profile.to_string())
            .expect("set the anchor profile");
        let rendered = expected(&catalog, &config, &BTreeMap::new());
        let report = judged(&github, &catalog, &rendered);
        assert!(
            !has_code(&report, "missing-index-entry") && has_code(&report, "unused"),
            "{profile}: the canonical link enrolls: {:?}",
            lines(report.errors.iter().chain(&report.warnings))
        );
        targets.push(rendered.index_targets);
    }
    assert_eq!(targets[0], targets[1], "one target under both profiles");
}

/// §AR-checker.1.4: inside each half the passes keep the single driver's relative
/// order, and the sort is stable on (path, line, message), so the merged order is
/// the single driver's exactly when no finding of one half ties one of the other.
/// Held across trees that raise findings in both halves.
#[test]
fn the_merge_has_no_cross_half_tie_and_is_the_stable_sort_of_both_halves() {
    let mut roots = vec![
        external_index_repo("merge_external_index"),
        kind_index_repo("merge_kind_index"),
        linked_repo("merge_linked"),
        drifted_include_repo("merge_drifted_include"),
    ];
    roots.extend(presentation_fixture("merge_presentation"));
    for root in roots {
        let (config, catalog) = loaded(&root);
        let none = BTreeMap::new();
        let workspace = CheckWorkspace::alone(&none);
        let rendered = expected(&catalog, &config, &none);
        let conformed = conform(
            config.schema(),
            &catalog,
            config.frame(),
            &TextOverlays::new(),
        );
        let judged = judge(
            config.rules(),
            config.schema(),
            &catalog,
            &rendered,
            config.frame(),
            &workspace,
        );
        let merged = check_on_disk(
            config.rules(),
            config.schema(),
            &catalog,
            &rendered,
            config.frame(),
            &workspace,
        );
        for (conformed, judged, merged) in [
            (&conformed.errors, &judged.errors, &merged.errors),
            (&conformed.warnings, &judged.warnings, &merged.warnings),
            (
                &conformed.suggestions,
                &judged.suggestions,
                &merged.suggestions,
            ),
        ] {
            let key = |d: &Diagnostic| (d.path.clone(), d.line, d.message.clone());
            let ties = judged
                .iter()
                .filter(|j| conformed.iter().any(|c| key(c) == key(j)))
                .collect::<Vec<_>>();
            assert!(
                ties.is_empty(),
                "{}: cross-half ties {:?}",
                root.display(),
                lines(ties)
            );
            let mut single = conformed.clone();
            single.extend(judged.iter().cloned());
            sort_diagnostics(&mut single);
            assert_eq!(lines(merged), lines(&single), "{}", root.display());
        }
    }
}
