//! Test module: the report gate (§FS-config-migrate.5.2). `check --suggestions`
//! on the tree with the file as read against the candidate: a candidate that
//! changes a finding or the exit is caught, and the two differences the gate
//! ignores — a finding located in the config, moved through the key map, and
//! the config-table names inside messages — pass. The records gate is
//! `config/migrate/tests_gates.rs`.

use std::path::{Path, PathBuf};

use super::config_migrate::{config_lines, describes_config, report_gate, surplus, v1_spelling};
use super::config_migrate_guidance::refusals as guidance;
use crate::config::Project;
use crate::config::migrate::{
    GateResult, Located, MigrateOutcome, Prepared, RefusalAnchor, translate,
};
use crate::model::ExpectedSection;
use crate::testing::{test_root, write};
use crate::writers::RootGuidance;
use crate::{MigrateOpts, config_migrate};

const FS: &str = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n";

/// A v1 tree with `config` and the files of `files` under a fresh root.
fn tree(name: &str, config: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = test_root(&format!("migrate-report-gate-{name}"));
    for (path, text) in files {
        write(&root.join(path), text);
    }
    write(&root.join("grund.toml"), config);
    root
}

/// `root`'s config translated: where it is, the v1 project, the candidate and
/// its key map.
fn prepared(root: &Path) -> (Located, Project, String, Vec<(usize, usize)>) {
    let located = Located {
        file: root.join("grund.toml"),
        root: root.to_path_buf(),
        config: "grund.toml".into(),
    };
    let Prepared::Translated {
        report,
        v1,
        line_map,
        ..
    } = translate(&located, false, false).expect("the v1 file reads")
    else {
        panic!("read as version 2");
    };
    assert_eq!(report.gates, Some(GateResult::Equal));
    let candidate = report.candidate.expect("a candidate");
    (located, *v1, candidate, line_map)
}

/// The v1 file of `body`, with `[scan] include = ["docs"]` unless `body` sets
/// it, since a missing default entry is refused (§FS-config-migrate.3.1.3).
fn v1_config(body: &str) -> String {
    let include = if body.contains("include =") {
        ""
    } else {
        "[scan]\ninclude = [\"docs\"]\n"
    };
    format!("grund_config_version = 1\n{body}{include}[id]\nnamed_sections = true\n")
}

/// §FS-config-migrate.5.2: a candidate that would change the findings — here
/// a marker that stops reading the one citation of `FS-a` — differs, with the
/// candidate's new warning marked `+ `.
#[test]
fn report_gate_catches_a_candidate_that_changes_the_findings() {
    let root = tree(
        "marker",
        &v1_config(&format!("{FS}[scan]\ninclude = [\"docs\", \"src\"]\n")),
        &[
            ("docs/FS-a.md", "# FS-a: A\n\nText.\n"),
            ("src/main.rs", "// Implements \u{a7}FS-a.\nfn main() {}\n"),
        ],
    );
    let (located, v1, candidate, map) = prepared(&root);
    assert!(report_gate(&located, &v1, &candidate, &map, "grund.toml").is_empty());
    let wrong = candidate.replacen("marker = \"§\"", "marker = \"@\"", 1);
    assert_ne!(wrong, candidate, "the candidate spells the marker");
    let lines = report_gate(&located, &v1, &wrong, &map, "grund.toml");
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("+ warning") && line.contains("never cited: FS-a")),
        "{lines:?}"
    );
}

/// A workspace root that opts out of itself and lists a version-2 member: the
/// run warns at a `grund.toml` line, and the message names that line again.
fn opted_out_root(name: &str) -> PathBuf {
    let config = v1_config(&format!(
        "[workspace]\nmembers = [\"sub\"]\ninclude_root = false\n{FS}"
    ));
    tree(
        name,
        &config,
        &[
            ("docs/FS-a.md", "# FS-a: A\n\nText.\n"),
            ("sub/grund.toml", "grund_config_version = 2\n"),
        ],
    )
}

/// §FS-config-migrate.5.2: a finding located in the config is compared at the
/// v1 line its key came from, in its location and in its message, though the
/// candidate puts the key on another line.
#[test]
fn report_gate_moves_a_config_finding_through_the_key_map() {
    let root = opted_out_root("config-finding");
    let (located, v1, candidate, map) = prepared(&root);
    assert!(
        map.iter().any(|(v2, v1)| v2 != v1),
        "the candidate moves a key: {map:?}"
    );
    let lines = report_gate(&located, &v1, &candidate, &map, "grund.toml");
    assert_eq!(lines, Vec::<String>::new());
    match config_migrate(&MigrateOpts {
        path: root,
        write: false,
        drop_comments: false,
    }) {
        Ok(MigrateOutcome::Migrated(report)) => assert_eq!(report.gates, Some(GateResult::Equal)),
        other => panic!("{other:?}"),
    }
}

/// §FS-config-migrate.5.2: the v1 run's `config-version-1` caution is the one
/// thing migrating ends, so a clean v1 project previews with no `differs:` even
/// though its own run carries the caution and the candidate's does not.
#[test]
fn report_gate_drops_the_version_1_caution() {
    let root = tree(
        "version-1-caution",
        &v1_config(FS),
        &[("docs/FS-a.md", "# FS-a: A\n\nText.\n")],
    );
    let (warnings, output) = super::check::check_with_projects(super::check::CheckOpts {
        path: root.clone(),
        path_provided: true,
        ..super::check::CheckOpts::default()
    });
    assert!(warnings.is_empty());
    let (output, _) = output.expect("the tree checks");
    assert!(
        output
            .report
            .warnings
            .iter()
            .any(|f| f.code == "config-version-1"),
        "the v1 run carries the caution the gate must drop"
    );
    let (located, v1, candidate, map) = prepared(&root);
    assert!(report_gate(&located, &v1, &candidate, &map, "grund.toml").is_empty());
    match config_migrate(&MigrateOpts {
        path: root,
        write: false,
        drop_comments: false,
    }) {
        Ok(MigrateOutcome::Migrated(report)) => assert_eq!(report.gates, Some(GateResult::Equal)),
        other => panic!("{other:?}"),
    }
}

/// §FS-config-migrate.5.2: "a finding with no mapped line is a difference".
#[test]
fn report_gate_counts_an_unmapped_config_finding_as_a_difference() {
    let root = opted_out_root("unmapped");
    let (located, v1, candidate, _) = prepared(&root);
    let lines = report_gate(&located, &v1, &candidate, &[], "grund.toml");
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("+ ") && line.contains("grund.toml")),
        "{lines:?}"
    );
}

/// §FS-config-migrate.5.2: the config-table names inside messages are spelled
/// as v1 on both sides.
#[test]
fn report_gate_spells_config_table_names_as_v1() {
    assert_eq!(
        v1_spelling("no file matches [schema.sources] exclude entry `x/`"),
        "no file matches [scan] exclude entry `x/`"
    );
    assert_eq!(
        v1_spelling("[rules.citations.FS] must names AR"),
        "[citations.FS] must names AR"
    );
    assert_eq!(v1_spelling("[schema] id_format"), "[id] format");
}

/// The `<config>:<line>` a message names is moved; another config's is not.
#[test]
fn report_gate_moves_only_this_configs_lines_inside_a_message() {
    let moved = config_lines(
        "grund.toml:5: x, see sub/grund.toml:5 and grund.toml:7",
        "grund.toml",
        |line| line - 1,
    );
    assert_eq!(
        moved,
        "grund.toml:4: x, see sub/grund.toml:5 and grund.toml:6"
    );
}

/// §FS-config-migrate.5.2: the projects checked are compared — a candidate that
/// checks the root as well as its member differs by that project.
#[test]
fn report_gate_compares_the_projects_checked() {
    let root = opted_out_root("projects");
    let (located, v1, candidate, map) = prepared(&root);
    let wrong = candidate.replacen("include_root = false", "include_root = true", 1);
    assert_ne!(wrong, candidate, "the candidate spells include_root");
    let lines = report_gate(&located, &v1, &wrong, &map, "grund.toml");
    assert!(
        lines.iter().any(|line| line.starts_with("+ project ")),
        "{lines:?}"
    );
}

/// §FS-config-migrate.5.2: the runs are compared as multisets, so a line twice
/// in one run and once in the other is one difference.
#[test]
fn report_gate_compares_the_runs_as_multisets() {
    let line = |text: &str| text.to_string();
    let before = [line("a"), line("a"), line("b")];
    let after = [line("b"), line("a")];
    assert_eq!(surplus(&before, &after, "- "), ["- a"]);
    assert_eq!(surplus(&after, &before, "+ "), Vec::<String>::new());
}

/// §FS-config-migrate.5.3: a v1 value row gains a declaration shape in version
/// 2, so the guidance comparison refuses at its `values` line even where no
/// `AGENTS.md` exists for the report gate to compare.
#[test]
fn guidance_refuses_a_value_row_with_no_agents_md() {
    let config = v1_config(&format!(
        "{FS}[[kinds]]\nkind = \"TERM\"\nfile = \"terms.md\"\nvalues = true\n[scan]\ninclude = [\"docs\", \"terms.md\"]\n"
    ));
    let root = tree(
        "values-guidance",
        &config,
        &[
            ("docs/FS-a.md", "# FS-a: A\n\nText.\n"),
            ("terms.md", "# Terms\n\n## TERM-a: A\n\nThe first term.\n"),
        ],
    );
    let opts = MigrateOpts {
        path: root.clone(),
        write: true,
        drop_comments: false,
    };
    let Ok(MigrateOutcome::Migrated(report)) = config_migrate(&opts) else {
        panic!("migrates");
    };
    let refusals: Vec<_> = report
        .refusals
        .iter()
        .map(|r| {
            (
                r.anchor,
                r.what.contains("`TERM` values = true"),
                r.details.len(),
            )
        })
        .collect();
    assert_eq!(refusals, [(RefusalAnchor::Line(8), true, 2)]);
    assert!(!report.wrote);
    assert!(std::fs::read_to_string(root.join("grund.toml")).is_ok_and(|text| text == config));
}

/// §FS-config-migrate.5.3: a compared section that differs for another cause
/// is one block with its lines, and a version alone is one block too.
#[test]
fn guidance_refuses_a_changed_section_or_version() {
    let section = |bytes: &str| ExpectedSection {
        heading: "### Citation directions".into(),
        noun: "citation directions".into(),
        bytes: bytes.into(),
    };
    let block = |version, bytes: &str| RootGuidance {
        version,
        sections: vec![section(bytes)],
    };
    let changed = guidance(&block(19, "a\nb\n"), &block(19, "a\nc\n"), &[]);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].details, ["- b", "+ c"]);
    assert!(changed[0].what.contains("its citation directions section"));
    let version = guidance(&block(19, "a\n"), &block(21, "a\n"), &[]);
    assert!(version[0].what.contains("from v19 to v21"), "{version:?}");
    assert!(guidance(&block(19, "a\n"), &block(19, "a\n"), &[]).is_empty());
}

/// §FS-config-migrate.5.2: the report gate leaves out a stale block whose part
/// describes the config, which a migration is expected to change, and still
/// compares one in the five generated sections the guidance gate holds
/// (§FS-config-migrate.5.3).
#[test]
fn report_gate_ignores_a_stale_block_outside_the_generated_sections() {
    let stale = |noun: &str| crate::Finding {
        severity: "error",
        code: "agents-init",
        path: Some("AGENTS.md".to_string()),
        line: Some(3),
        column: None,
        message: format!(
            "repo maintenance: stale grund init block: the {noun} section differs from grund.toml \
             — run `grund init` to refresh (does not affect citation validity)"
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    };
    for noun in [
        "citation scheme",
        "project map",
        "declarations and citations",
        "rules",
    ] {
        assert!(describes_config(&stale(noun)), "{noun}");
    }
    for noun in crate::model::GENERATED_SECTIONS {
        assert!(!describes_config(&stale(noun)), "{noun}");
    }
    let mut outdated = stale("rules");
    outdated.message = "repo maintenance: outdated grund init block v35 — run `grund init` to \
                        update to v39 (does not affect citation validity)"
        .into();
    assert!(!describes_config(&outdated));
}

/// §FS-config-migrate.5.2, §FS-config-migrate.5.3: a v1 tree `init` wrote a
/// block for migrates with both gates equal, whatever its candidate renders
/// into the block's citation scheme.
#[test]
fn a_block_init_wrote_passes_both_gates() {
    let root = tree(
        "initialized-block",
        &v1_config(FS),
        &[("docs/FS-a.md", "# FS-a: A\n\nText.\n")],
    );
    crate::writers::init(crate::writers::InitOpts {
        target: root.clone(),
        no_vcs: true,
        ..crate::writers::InitOpts::default()
    })
    .expect("init");
    let (located, v1, candidate, map) = prepared(&root);
    assert_eq!(
        report_gate(&located, &v1, &candidate, &map, "grund.toml"),
        Vec::<String>::new()
    );
    match config_migrate(&MigrateOpts {
        path: root,
        write: false,
        drop_comments: false,
    }) {
        Ok(MigrateOutcome::Migrated(report)) => {
            assert_eq!(report.gates, Some(GateResult::Equal));
            assert!(report.refusals.is_empty(), "{:?}", report.refusals);
        }
        other => panic!("{other:?}"),
    }
}
