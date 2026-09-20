//! Test module: companion agent-entrypoint discovery and validation (§AR-core-module-layout.1.5)

use std::fs;

use super::InitAgentEntrypointSelection;
use super::init_plan::{
    requested_init_companion_agent_entrypoints, workspace_init_companion_agent_entrypoints,
};
use crate::checker::check_findings;
use crate::config::Config;
use crate::scanner::{
    AgentEntrypoint, CanonicalSurfaceReach, InitCompanionAgentEntrypoint,
    agents_with_own_entrypoint, companion_agent_entrypoints, scan_tree,
};
use crate::testing::{current_block, test_root, write};

#[test]
fn discovers_known_companion_agent_entrypoints() {
    let root = test_root("discovers_known_companion_agent_entrypoints");
    write(&root.join("AGENTS.override.md"), "# Codex override notes\n");
    write(&root.join("CLAUDE.md"), "# Claude notes\n");
    write(&root.join(".claude/CLAUDE.md"), "# Claude project notes\n");
    write(&root.join("GEMINI.md"), "# Gemini notes\n");
    write(&root.join(".pi/AGENTS.md"), "# Pi notes\n");
    write(
        &root.join(".github/copilot-instructions.md"),
        "# Copilot notes\n",
    );

    let companions = companion_agent_entrypoints(&root).expect("discover companions");
    let rels = companions
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        rels,
        vec![
            "AGENTS.override.md",
            "CLAUDE.md",
            ".claude/CLAUDE.md",
            "GEMINI.md",
            ".pi/AGENTS.md",
            ".github/copilot-instructions.md"
        ]
    );
}

#[test]
fn init_discovers_missing_aliases_for_existing_agent_workspaces() {
    // §FS-init.2.1.1.2: `.claude/` proves Claude is in use, which is one fact
    // and so one alias — the root-visible `CLAUDE.md`, not both of Claude's
    // entrypoints.
    let root = test_root("init_discovers_missing_aliases_for_existing_agent_workspaces");
    fs::create_dir_all(root.join(".claude")).expect("create .claude");
    fs::create_dir_all(root.join(".gemini")).expect("create .gemini");
    fs::create_dir_all(root.join(".pi")).expect("create pi");
    fs::create_dir_all(root.join(".github/workflows")).expect("create github metadata");

    let companions =
        workspace_init_companion_agent_entrypoints(&root, CanonicalSurfaceReach::EveryEntrypoint)
            .expect("discover workspace aliases");
    let rels = companions
        .iter()
        .map(|entrypoint| match entrypoint {
            InitCompanionAgentEntrypoint::Existing(path)
            | InitCompanionAgentEntrypoint::MissingAlias(path) => path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
        })
        .collect::<Vec<_>>();

    assert_eq!(rels, vec!["CLAUDE.md", "GEMINI.md", ".pi/AGENTS.md"]);
}

/// Answering *does this agent have an entrypoint* on looser evidence than
/// the update set and `grund check` use is how the two come to disagree
/// about what an entrypoint is.
#[test]
fn an_unclaimed_generic_file_is_not_its_agent_s_entrypoint() {
    // §FS-init.2.1.1 / §FS-init.2.1.2: `.rules` is too generic to attribute to
    // Zed by filename alone, so a build-rules file that no `.zed/` and no
    // managed block claims is somebody else's (§FS-check.3.5.1).
    let root = test_root("an_unclaimed_generic_file_is_not_its_agent_s_entrypoint");
    write(&root.join(".rules"), "# somebody else's build rules\n");

    let covered = agents_with_own_entrypoint(&root, CanonicalSurfaceReach::EveryEntrypoint)
        .expect("inspect entrypoints");
    assert!(
        !covered.contains(&AgentEntrypoint::Zed),
        "an unclaimed .rules is not Zed's entrypoint"
    );

    // The `.zed/` workspace is the evidence that settles it.
    fs::create_dir_all(root.join(".zed")).expect("create .zed");
    let covered = agents_with_own_entrypoint(&root, CanonicalSurfaceReach::EveryEntrypoint)
        .expect("inspect entrypoints");
    assert!(
        covered.contains(&AgentEntrypoint::Zed),
        "with .zed/ present the same file is Zed's entrypoint"
    );
}

#[test]
fn init_requests_one_entrypoint_per_agent() {
    // §FS-init.2.1.1: an explicit flag updates every entrypoint the agent has
    // and creates one only for an agent that has none — the block is the same
    // bytes in each, so a second file is the same guidance read twice.
    let root = test_root("init_requests_one_entrypoint_per_agent");
    write(&root.join(".claude/CLAUDE.md"), "# Claude project notes\n");
    let selection = InitAgentEntrypointSelection {
        claude: true,
        gemini: true,
        ..InitAgentEntrypointSelection::default()
    };

    let (canonical_symlinks, companions) = requested_init_companion_agent_entrypoints(
        &root,
        &selection,
        CanonicalSurfaceReach::EveryEntrypoint,
    )
    .expect("select requested");

    assert!(canonical_symlinks.is_empty());
    let planned = companions
        .iter()
        .map(|entrypoint| {
            let rel = entrypoint
                .path()
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let created = matches!(entrypoint, InitCompanionAgentEntrypoint::MissingAlias(_));
            (rel, created)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        planned,
        vec![
            (".claude/CLAUDE.md".to_string(), false),
            ("GEMINI.md".to_string(), true),
        ],
        "the Claude entrypoint on disk should be updated and no second one created"
    );
}

#[test]
fn check_ignores_companion_agent_entrypoints_without_canonical_agents_md() {
    let root = test_root("check_ignores_companion_agent_entrypoints_without_canonical_agents_md");
    write(&root.join("CLAUDE.md"), "# Project agent notes\n");
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        report
            .errors
            .iter()
            .all(|error| error.code != "agents-init"),
        "project-owned AGENTS.md should not require a managed block without canonical AGENTS.md"
    );
}

/// §FS-init.2.3.7.2: agent-entrypoint validation reads the marker line and the
/// version it carries, not a byte-diff against the canonical text — the body
/// here is arbitrary, and the finding is about `v99` alone.
#[test]
fn check_validates_managed_companion_without_canonical_agents_md() {
    let root = test_root("check_validates_managed_companion_without_canonical_agents_md");
    write(
        &root.join("CLAUDE.md"),
        "## Grounding with grund (v99)\n\nold block\n",
    );
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let expected_path = root.join("CLAUDE.md");

    assert!(
        report.errors.iter().any(|error| error.code == "agents-init"
            && error.path.as_deref() == Some(expected_path.as_path())
            && error.message.contains("unsupported grund init block v99")),
        "managed companion entrypoint should be version-checked without AGENTS.md: {:?}",
        report
            .errors
            .iter()
            .map(|error| (&error.path, &error.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn check_validates_managed_zed_rules_without_canonical_agents_md() {
    // §FS-check.3.5.1 / §FS-init.2.1.2: `.rules` is not discovered by filename
    // alone, but a managed block proves it is a grund-owned Zed companion
    // and must still get init-block drift detection.
    let root = test_root("check_validates_managed_zed_rules_without_canonical_agents_md");
    write(
        &root.join(".rules"),
        "## Grounding with grund (v99)\n\nold block\n",
    );
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let expected_path = root.join(".rules");

    assert!(
        report.errors.iter().any(|error| error.code == "agents-init"
            && error.path.as_deref() == Some(expected_path.as_path())
            && error.message.contains("unsupported grund init block v99")),
        "managed .rules should be version-checked without AGENTS.md: {:?}",
        report
            .errors
            .iter()
            .map(|error| (&error.path, &error.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn check_validates_zed_workspace_rules_when_canonical_exists() {
    // §FS-check.3.5.1 / §FS-init.2.1.2: in a Zed workspace, `.rules` is owned
    // by the Zed companion path and must be validated when AGENTS.md exists.
    let root = test_root("check_validates_zed_workspace_rules_when_canonical_exists");
    write(&root.join("AGENTS.md"), &current_block());
    write(&root.join(".zed/settings.json"), "{}\n");
    write(
        &root.join(".rules"),
        "# Zed notes without a managed block\n",
    );
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let expected_path = root.join(".rules");

    assert!(
        report.errors.iter().any(|error| error.code == "agents-init"
            && error.path.as_deref() == Some(expected_path.as_path())
            && error.message.contains("missing grund init block v10")),
        "Zed workspace .rules should be required to carry the managed block: {:?}",
        report
            .errors
            .iter()
            .map(|error| (&error.path, &error.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn check_ignores_unmanaged_generic_rules_without_zed_workspace() {
    // §FS-init.2.1.2: `.rules` is too generic to attribute to Zed by file
    // existence alone, so a generic unmanaged file outside a `.zed/`
    // workspace must not become a companion check target.
    let root = test_root("check_ignores_unmanaged_generic_rules_without_zed_workspace");
    write(&root.join("AGENTS.md"), &current_block());
    write(&root.join(".rules"), "# Build rules, not Zed\n");
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let generic_rules = root.join(".rules");

    assert!(
        report.errors.iter().all(|error| {
            error.code != "agents-init" || error.path.as_deref() != Some(generic_rules.as_path())
        }),
        "generic .rules must not be validated as a Zed companion: {:?}",
        report
            .errors
            .iter()
            .map(|error| (&error.path, &error.message))
            .collect::<Vec<_>>()
    );
}
