//! Test module: the guidance gate's one grammar (§FS-config-migrate.5.3) and
//! the report gate's matching drop (§FS-config-migrate.5.2). A citation
//! directions section that differs only in the grounding setting's spelling
//! passes, and the root block's stale finding on it, `AGENTS.md`'s or a
//! companion's, leaves both runs; a scoped block's stays. The byte comparison
//! itself is `tests_config_migrate_gates.rs`.
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use super::config_migrate::root_directions;
use super::config_migrate_guidance::{refusals, spelled_apart};
use crate::config::migrate::{GateResult, MigrateOutcome};
use crate::model::ExpectedSection;
use crate::testing::{test_root, write};
use crate::writers::{ComparedGuidance, RootGuidance};
use crate::{MigrateOpts, config_migrate};

/// One citation directions bullet for `e2e` that grounds nothing, its setting
/// spelled in config `version`.
fn bullet(version: u32) -> String {
    let setting =
        crate::config::grounding_setting("e2e", version, |fragment| format!("`{fragment}`"));
    format!(
        "- Each file in **tests/e2e/** that cites anything must cite FS. A file there that \
         cites nothing is not checked; {setting} makes it a finding.\n"
    )
}

/// A config's compared guidance: its live citation directions, and the ones
/// compared, the grounding setting spelled in the candidate's grammar.
fn side(grammar: u32, live: &str, compared: &str) -> ComparedGuidance {
    ComparedGuidance {
        grammar,
        guidance: RootGuidance {
            version: 47,
            sections: vec![ExpectedSection {
                heading: "### Citation directions".into(),
                noun: "citation directions".into(),
                bytes: compared.into(),
            }],
        },
        live_directions: live.into(),
    }
}

/// §FS-config-migrate.5.3: a section whose bytes differ only in the
/// setting's spelling is equal in one grammar, so it is no refusal and the gate
/// says it passed it in one grammar; a section equal as bytes, or one that still
/// differs in one grammar, is not reported as passed that way.
#[test]
fn the_setting_spelling_alone_passes_in_one_grammar() {
    let (v1, v2) = (bullet(1), bullet(2));
    assert_ne!(v1, v2, "the two grammars spell the setting apart");
    let before = side(1, &v1, &v2);
    let after = side(2, &v2, &v2);
    assert!(refusals(&before.guidance, &after.guidance, &[]).is_empty());
    assert!(spelled_apart(&before, &after));
    // Equal as bytes: nothing was set aside.
    assert!(!spelled_apart(&side(2, &v2, &v2), &after));
    // A real difference stays one, whatever the spelling.
    let moved = v2.replace("tests/e2e/", "tests/");
    let before = side(1, &v1.replace("tests/e2e/", "tests/"), &moved);
    assert!(!spelled_apart(&before, &after));
    assert_eq!(refusals(&before.guidance, &after.guidance, &[]).len(), 1);
}

/// §FS-config-migrate.5.2: only the root block's stale citation directions
/// finding is the one the one-grammar comparison passed: in `AGENTS.md` or a
/// companion that carries the root block, never a scoped block's, and never
/// another section's.
#[test]
fn only_the_root_blocks_directions_are_dropped() {
    let stale = |path: &str, noun: &str| crate::Finding {
        severity: "error",
        code: "agents-init",
        path: Some(path.to_string()),
        line: Some(3),
        column: None,
        message: format!(
            "repo maintenance: stale grund init block: the {noun} section differs from grund.toml \
             — run `grund init` to refresh (does not affect citation validity)"
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    };
    assert!(root_directions(&stale("AGENTS.md", "citation directions")));
    assert!(root_directions(&stale("CLAUDE.md", "citation directions")));
    assert!(!root_directions(&stale(
        "tests/e2e/AGENTS.md",
        "citation directions"
    )));
    assert!(!root_directions(&stale("AGENTS.md", "chapter rules")));
}

/// The issue's version-1 tree: a non-citable `e2e` folder with a `must` rule
/// that grounds nothing, `init` run over it, and a `CLAUDE.md` that is a file
/// of its own, so its block is compared beside `AGENTS.md`'s (§FS-check.3.5).
fn ungrounded_tree(name: &str) -> crate::testing::TemporaryRoot {
    let root = test_root(&format!("migrate-one-grammar-{name}"));
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n\n\
         [[kinds]]\nkind = \"e2e\"\nfolder = \"tests/e2e\"\ncitable = false\n\n\
         [scan]\ninclude = [\"docs\"]\n\n[citations.e2e]\nmust = [\"FS\"]\n",
    );
    write(&root.join("docs/fs.md"), "# FS-login: Login\n\nText.\n");
    write(&root.join("tests/e2e/login.md"), "Cites \u{a7}FS-login.\n");
    write(&root.join("CLAUDE.md"), "# Notes\n");
    crate::writers::init(crate::writers::InitOpts {
        target: root.clone(),
        no_vcs: true,
        ..crate::writers::InitOpts::default()
    })
    .expect("init");
    root
}

/// §FS-config-migrate.5.3, §FS-config-migrate.5.2: the tree migrates with
/// both gates equal, though `AGENTS.md` and the `CLAUDE.md` beside it each
/// spell the setting in version 1 and the candidate's run reports both stale.
#[test]
fn a_root_block_and_its_companion_migrate_in_one_grammar() {
    let root = ungrounded_tree("companion");
    let claude = std::fs::read_to_string(root.join("CLAUDE.md")).expect("CLAUDE.md");
    assert!(claude.contains("`require_grounding = true`"), "{claude}");
    match config_migrate(&MigrateOpts {
        path: root.to_path_buf(),
        write: false,
        drop_comments: false,
    }) {
        Ok(MigrateOutcome::Migrated(report)) => {
            assert!(report.refusals.is_empty(), "{:?}", report.refusals);
            assert_eq!(report.gates, Some(GateResult::Equal));
        }
        other => panic!("{other:?}"),
    }
}
