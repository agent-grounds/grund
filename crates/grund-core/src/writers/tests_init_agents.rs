//! Test module: init scaffolding and the managed agent block (§FS-init)

use std::path::{Path, PathBuf};

use super::init::init_fs_home;
use super::init_block::{AgentsUpdateResult, update_agents_text};
use super::init_render::render_agents_md;
use super::*;
use crate::checker::check_findings;
use crate::config::{Config, load_config};
use crate::scanner::scan_tree;
use crate::templates::{AGENT_SETUP_INSTRUCTIONS, canonical_template_text, render_grund_toml};
use crate::testing::{current_block, current_marker, test_root, write};

/// §FS-init.5.3: the distributable skill and the binary-embedded copy the CLI
/// prints must be byte-identical, and a release that edits one surface
/// without the other is invalid. Nothing enforced that, so an edit to the
/// repository copy alone shipped stale setup instructions to every agent
/// reaching grund through `agent-setup-instructions` rather than the repo.
#[test]
fn agent_setup_instructions_match_the_distributable_skill() {
    let distributable =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/grund-init/SKILL.md");
    // Absent when the tests run from a packaged crate rather than the
    // workspace; in the repository — where the invariant can be violated —
    // it is always present.
    let Ok(text) = std::fs::read_to_string(&distributable) else {
        return;
    };
    assert_eq!(
        text, AGENT_SETUP_INSTRUCTIONS,
        "skills/grund-init/SKILL.md and crates/grund-core/assets/skills/grund-init/SKILL.md must be byte-identical (§FS-init.5.3)"
    );
}

/// §FS-init.5.1: what the instructions *say*, as against the byte-identity of
/// the two copies above. They read as an agent skill: inspect the target repo
/// and its existing artifacts first, recommend with repo evidence and pros and
/// cons, ask the user to confirm or override, write `grund.toml` **before**
/// `grund init` so the generated managed block reflects the chosen grammar, and
/// only then validate with `grund config validate` and `grund check`. The three
/// offsets are the ordering claim — a workflow with the right steps in the
/// wrong order writes a block against a grammar nobody chose.
#[test]
fn agent_setup_instructions_name_the_ordered_adoption_workflow() {
    let text = AGENT_SETUP_INSTRUCTIONS;
    for phrase in [
        "Inspect the target repo before asking questions",
        "roadmaps, changelogs, decisions, plans, tests, and agent instruction files",
        "repo evidence",
        "- Pros.",
        "- Cons.",
        "confirm or override every option",
        "grund config validate",
        "grund check",
    ] {
        assert!(
            text.contains(phrase),
            "the setup instructions must name {phrase:?}"
        );
    }

    let write_config = text
        .find("Write `grund.toml` from the analysis")
        .expect("the instructions write the config");
    let run_init = text
        .find("Run `grund init [path]")
        .expect("the instructions run init");
    let validate = text
        .find("Run `grund config validate [path]`")
        .expect("the instructions validate");
    assert!(
        write_config < run_init,
        "the config must be written before `grund init`, or the managed block reflects a grammar nobody chose"
    );
    assert!(
        run_init < validate,
        "validation is what the run is checked by, so it comes last"
    );
}

/// §FS-init.5.2: adopting a docs-heavy repository is a choice made *before* any
/// write — the canonical artifact types are shown beside what the repository
/// already has, the three adoption models are offered, and the recommended
/// `grund init` form reaches for `--docs` only where a scaffold is wanted.
#[test]
fn agent_setup_instructions_offer_the_three_adoption_models_before_any_write() {
    let text = AGENT_SETUP_INSTRUCTIONS;
    assert!(
        text.contains(
            "canonical `grund`, canonical core plus project-specific extras, or existing structure with citations"
        ),
        "the three adoption models are the choice the user is asked to make"
    );
    assert!(
        text.contains(
            "adding `--docs` only when the repo is fresh or the user selected a canonical-layout migration"
        ),
        "existing specs are represented in the config rather than replaced by generic scaffold folders"
    );

    let show_beside = text
        .find("show the canonical `grund` artifact types beside the detected")
        .expect("the instructions show the canonical types beside the detected ones");
    let write_config = text
        .find("Write `grund.toml` from the analysis")
        .expect("the instructions write the config");
    assert!(
        show_beside < write_config,
        "the adoption choice is made before config or docs are written"
    );
}

#[test]
fn embedded_templates_are_lf_canonical() {
    assert_eq!(
        canonical_template_text("alpha\r\nbeta\rgamma\n"),
        "alpha\nbeta\ngamma\n"
    );

    let config = Config::default_for(PathBuf::from("."));
    assert!(!render_agents_md("demo", &config, Path::new("."), true).contains('\r'));
    assert!(!render_grund_toml("demo", None).contains('\r'));
    assert!(!canonical_template_text(AGENT_SETUP_INSTRUCTIONS).contains('\r'));
    let fs_home = init_fs_home(&config);
    for (_, contents) in docs_scaffold(&fs_home) {
        assert!(!contents.contains('\r'));
    }
}

/// §FS-init.2.3.8.1: the `[id].section_separator` is one of the things the
/// effective config fills into the block rather than the `vN` text fixing — so
/// a repository that separates sections with `#` reads its own separator back
/// out of the rendered examples.
#[test]
fn agents_guidance_uses_configured_section_separator() {
    let mut config = Config::default_for(PathBuf::from("."));
    config.section_separator = "#".to_string();

    let rendered = render_agents_md("demo", &config, Path::new("."), true);

    assert!(
        rendered.contains("§<ID>#1` / `§<ID>#1.1"),
        "section examples should use the configured outer separator: {rendered}"
    );
    assert!(
        !rendered.contains("§<ID>.1` / `§<ID>.1.1"),
        "section examples must not hard-code dot as the outer separator"
    );
}

/// A configured kind title is the prose in its generated Project-map row
/// (§FS-config.3.4.3); the kind handle and home remain the navigational parts.
#[test]
fn project_map_uses_the_configured_kind_title() {
    let root = test_root("project_map_uses_the_configured_kind_title");
    write(
        &root.join("grund.toml"),
        "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\ntitle = \"Product contracts\"\n",
    );
    let config = load_config(&root).expect("load configured kind title");
    let rendered = render_agents_md("demo", &config, &root, true);
    assert!(
        rendered.contains("- [FS](docs/specs): Product contracts"),
        "{rendered}"
    );
}

#[test]
fn agents_update_appends_managed_block_when_missing() {
    let (updated, result) =
        update_agents_text("# Existing agents\n", &current_block(), "AGENTS.md")
            .expect("append block");

    assert_eq!(result, AgentsUpdateResult::Appended);
    assert!(updated.starts_with("# Existing agents\n\n"));
    assert_eq!(updated.matches(current_marker()).count(), 1);
}

#[test]
fn agents_update_does_not_append_current_block_twice() {
    // §FS-init.2.2: a file already holding the current rendered block is left
    // untouched (`Unchanged` → `exists `), not rewritten and reported `updated `.
    let existing = current_block();
    let (updated, result) =
        update_agents_text(&existing, &current_block(), "AGENTS.md").expect("current block");

    assert_eq!(result, AgentsUpdateResult::Unchanged);
    assert_eq!(updated, existing);
    assert_eq!(updated.matches(current_marker()).count(), 1);
}

/// §FS-init.2.3.10.1: a file already holding a supported block is re-rendered
/// and, where the render differs from the managed region on disk, only those
/// bytes are replaced — the content before the block stays byte-identical and
/// the run reports `Updated`, without `--force`.
#[test]
fn agents_update_rewrites_current_block_from_rendered_template() {
    // A block that differs from the current render (here: an extra hand-added
    // line inside the delimiters) is replaced and reported `Updated`.
    let mut stale = current_block();
    let insert_at = stale
        .find("<!-- END GRUND MANAGED BLOCK -->")
        .expect("rendered block carries the END delimiter");
    stale.insert_str(insert_at, "hand-edited line\n");
    let existing = format!("# Local notes\n\n{stale}");

    let (updated, result) = update_agents_text(&existing, &current_block(), "AGENTS.md")
        .expect("rewrite current block");

    assert_eq!(result, AgentsUpdateResult::Updated);
    assert!(updated.starts_with("# Local notes\n\n"));
    assert!(!updated.contains("hand-edited line"));
    assert_eq!(updated.matches(current_marker()).count(), 1);
}

/// §FS-init.3.2: the delimiters are the ownership boundary, so everything
/// before and after the block — and the block's position between them — is
/// preserved byte-for-byte on an update that is not a `--force` rewrite.
#[test]
fn agents_update_keeps_current_block_in_middle_position() {
    // §FS-init.2.3.1 / §FS-init.2.2: a block already current and already
    // sitting between user-authored sections is left byte-for-byte untouched
    // (`Unchanged` → `exists `) — nothing around it moves, nothing is rewritten.
    let existing = format!("# Existing agents\n\n{}\n# Local notes\n", current_block());
    let (updated, result) = update_agents_text(&existing, &current_block(), "AGENTS.md")
        .expect("non-EOF current block");

    assert_eq!(result, AgentsUpdateResult::Unchanged);
    assert_eq!(
        updated, existing,
        "an already-current block preserves every byte, inside and out"
    );
    assert!(updated.starts_with("# Existing agents\n\n"));
    assert!(updated.ends_with("\n# Local notes\n"));
    assert_eq!(updated.matches(current_marker()).count(), 1);
}

#[test]
fn agents_update_handles_crlf_line_endings() {
    // §FS-init.2.3.2: a CRLF-encoded AGENTS.md whose managed block is stale
    // (same version, different body) must still be detected and rewritten,
    // with the surrounding CRLF preserved verbatim.
    let existing = format!(
        "# Existing agents\r\n\r\n{}\r\n\r\nstale body line\r\n\r\n# Local notes\r\n",
        current_marker()
    );
    let (updated, result) = update_agents_text(&existing, &current_block(), "AGENTS.md")
        .expect("update CRLF stale block");

    assert_eq!(result, AgentsUpdateResult::Updated);
    assert!(
        updated.starts_with("# Existing agents\r\n\r\n"),
        "CRLF prefix must be preserved verbatim"
    );
    assert!(
        updated.ends_with("\n# Local notes\r\n"),
        "CRLF suffix must be preserved verbatim"
    );
    assert_eq!(updated.matches(current_marker()).count(), 1);
    assert!(!updated.contains("stale body line"));
}

/// §FS-init.2.3.9.2: a v3-and-earlier block predates the delimiters, so the H2
/// heading opens it and the next H1/H2 closes it. `init` still recognizes that
/// form and migrates it to the delimited one in place, reported `Updated`.
#[test]
fn agents_update_migrates_legacy_block_to_delimited_form() {
    // §FS-init.2.3.9 / §DF-managed-block-delimiters: a legacy H2-bounded block
    // sandwiched between user sections is replaced in place by the delimited
    // render, with both neighbors byte-identical.
    let existing =
        "# Existing agents\n\n## Grounding with grund (v3)\n\nlegacy body\n\n## Local notes\n";
    let (updated, result) =
        update_agents_text(existing, &current_block(), "AGENTS.md").expect("migrate legacy block");

    assert_eq!(result, AgentsUpdateResult::Updated);
    assert!(updated.starts_with("# Existing agents\n\n<!-- BEGIN GRUND MANAGED BLOCK -->\n"));
    assert!(updated.ends_with("<!-- END GRUND MANAGED BLOCK -->\n\n## Local notes\n"));
    assert!(!updated.contains("legacy body"));
    assert_eq!(updated.matches(current_marker()).count(), 1);
}

#[test]
fn agents_update_preserves_non_heading_content_after_delimited_block() {
    // §FS-init.2.3.9 / §DF-managed-block-delimiters: the managed region ends at
    // the END delimiter, so a third-party managed marker right after the
    // block — not an H1/H2, invisible to the legacy boundary — survives.
    let existing = format!(
        "{}\n<!-- rhei:begin -->\nother tool's region\n<!-- rhei:end -->\n",
        current_block()
    );
    let (updated, result) = update_agents_text(&existing, &current_block(), "AGENTS.md")
        .expect("update delimited block");

    assert_eq!(result, AgentsUpdateResult::Unchanged);
    assert!(updated.contains("<!-- rhei:begin -->\nother tool's region\n<!-- rhei:end -->\n"));
}

/// §FS-init.2.3.10.2: an entrypoint carrying a block version this binary does
/// not support is a refusal, not a downgrade — the splice never runs, so the
/// caller has the original text to leave on disk unchanged, and the message
/// names the version it found beside the one it supports.
#[test]
fn agents_update_refuses_a_newer_block_version() {
    let existing = "# Local notes\n\n<!-- BEGIN GRUND MANAGED BLOCK -->\n\
         ## Grounding with grund (v99)\n\nbody from a newer grund\n\
         <!-- END GRUND MANAGED BLOCK -->\n";
    let err = update_agents_text(existing, &current_block(), "AGENTS.md")
        .expect_err("a newer block version must refuse the update");
    let message = format!("{err:#}");
    assert!(
        message.contains("AGENTS.md contains newer grund init block v99"),
        "the refusal names the file and the version it found: {message}"
    );
}

#[test]
fn agents_update_refuses_malformed_delimiters() {
    // §FS-init.2.3.11: splicing against broken delimiters risks eating user
    // content, so init errors out and leaves the text alone.
    for (existing, defect) in [
        (
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v4)\n\nbody\n",
            "missing `<!-- END GRUND MANAGED BLOCK -->`",
        ),
        (
            "notes\n\n<!-- END GRUND MANAGED BLOCK -->\n",
            "`<!-- END GRUND MANAGED BLOCK -->` without a begin delimiter",
        ),
        (
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n<!-- BEGIN GRUND MANAGED BLOCK -->\n<!-- END GRUND MANAGED BLOCK -->\n",
            "duplicate `<!-- BEGIN GRUND MANAGED BLOCK -->`",
        ),
        (
            "<!-- BEGIN GRUND MANAGED BLOCK -->\nbody without a version heading\n<!-- END GRUND MANAGED BLOCK -->\n",
            "no `## Grounding with grund (vN)` heading between the delimiters",
        ),
    ] {
        let err = update_agents_text(existing, &current_block(), "AGENTS.md")
            .expect_err("malformed delimiters must refuse the update");
        let message = format!("{err:#}");
        assert!(
            message.contains("malformed grund managed block") && message.contains(defect),
            "unexpected error for {existing:?}: {message}"
        );
    }
}

#[test]
fn check_reports_malformed_agents_block() {
    // §FS-check.3.5.2: broken delimiters are an agents-init error anchored at
    // the offending delimiter line, and the file is never rewritten.
    let root = test_root("check_reports_malformed_agents_block");
    write(
        &root.join("AGENTS.md"),
        "# Title\n\n<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v4)\n\nbody\n",
    );
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        report.errors.iter().any(|error| error.code == "agents-init"
            && error.line == Some(3)
            && error.message.contains(
                "malformed grund managed block: missing `<!-- END GRUND MANAGED BLOCK -->`"
            )),
        "malformed delimiters should be a line-anchored agents-init error: {:?}",
        report
            .errors
            .iter()
            .map(|error| (&error.line, &error.message))
            .collect::<Vec<_>>()
    );
}

/// §FS-init.2.3.8.2: the worked example citation is written in the escaped
/// illustration form, never as a live citation — its ID is deliberately not a
/// declaration of the host repo, so a live marker would dangle.
#[test]
fn rendered_block_citation_example_is_escaped() {
    // §FS-init.2.3.8: the worked example must be the `<§>`-escaped illustration
    // form — a live `§` would make freshly generated output fail the host
    // repo's own `grund check` as a dangling reference.
    let block = current_block();
    assert!(
        block.contains("`<§>FS-042-user-login.3.1`"),
        "worked example should be escaped: {block}"
    );
    assert!(
        !block.contains("`§FS-042-user-login"),
        "worked example must not be a live citation: {block}"
    );
}
