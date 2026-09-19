//! The scaffold payload `grund init` writes (§FS-init.2.1): the reference
//! templates, embedded in the binary from `templates/` in the source tree, the
//! newline canonicalization every one of them is read through, and the
//! `grund.toml` render that is nothing but a name substituted into one of them
//! (§FS-init.2.4).
//!
//! Data rather than a decision, which is why it is here and not in the run:
//! `writers/init.rs` decides *which* of these a `--docs` scaffold writes and
//! where, and this file only says what each one says (§AR-system.2.11).

use crate::config::escape_toml_basic;

pub(super) const AGENTS_TEMPLATE: &str = include_str!("../../assets/templates/AGENTS.md");
/// The scaffold config. Its `[citations]` block comment explains the five levels
/// and hands the reader the `CITATION_DIRECTIONS_URL` of
/// `citation_directions.rs`, where it used to cite
/// §FS-config.3.9 by ID: the file lands verbatim in the adopting repository, so
/// an ID of this one names a document that reader does not have — or an
/// unrelated one of their own (§REQ-shipped-surfaces.1).
const GRUND_TOML_TEMPLATE: &str = include_str!("../../assets/templates/grund.toml");
pub(crate) const GRUND_DOC_TEMPLATE: &str = include_str!("../../assets/templates/grund.md");
pub(crate) const GOALS_TEMPLATE: &str = include_str!("../../assets/templates/goals.md");
pub(crate) const REQUIREMENTS_TEMPLATE: &str =
    include_str!("../../assets/templates/requirements.md");
pub(crate) const FS_README_TEMPLATE: &str =
    include_str!("../../assets/templates/functional-spec-README.md");
pub(crate) const E2E_README_TEMPLATE: &str = include_str!("../../assets/templates/e2e-README.md");
pub(crate) const AS_README_TEMPLATE: &str =
    include_str!("../../assets/templates/architecture-README.md");
pub(crate) const DF_README_TEMPLATE: &str =
    include_str!("../../assets/templates/decisions-functional-README.md");
pub(crate) const DA_README_TEMPLATE: &str =
    include_str!("../../assets/templates/decisions-architectural-README.md");
pub(crate) const GITKEEP_TEMPLATE: &str = include_str!("../../assets/templates/gitkeep.md");
/// The setup skill, printed byte-for-byte by `grund agent-setup-instructions`
/// into whatever repository the agent is standing in (§FS-init.5.3). That is why
/// `skills/` is an unwalked home (§FS-config.3.4.7.1) and why nothing in the file
/// cites an ID of this repository: it would name a document the reader does not
/// have (§REQ-shipped-surfaces.1), so its links are the public URLs of the pages
/// they used to cite. What it teaches is grounded here instead — the config
/// walkthrough ends on the full-tree scope that reports what `[scan] include`
/// left out (§FS-check.1.3), its `[citations]` section is the canonical
/// citation-directions page verbatim (§FS-config.3.9, kept in sync by the
/// asset-sync check), and its setup step covers the clickable integrations
/// (§FS-integrations), the global agent instruction files `--write`
/// synchronizes (§FS-integrations.4.3.8), and the committable repository opinion
/// with its per-agent gate (§DF-repo-conversation-opinion,
/// §DF-conversation-link-target.2.4).
pub const AGENT_SETUP_INSTRUCTIONS: &str = include_str!("../../assets/skills/grund-init/SKILL.md");

pub fn canonical_template_text(template: &str) -> String {
    template.replace("\r\n", "\n").replace('\r', "\n")
}

/// The generated `grund.toml` — every default written out explicitly as a
/// teaching surface, with only `project_name` substituted (§FS-init.2.4.3). With
/// `--description`, the commented `project_description` teaching line becomes
/// the real key (§FS-init.2.4.7, §DF-workspace-member-descriptions).
pub(crate) fn render_grund_toml(name: &str, description: Option<&str>) -> String {
    let mut rendered =
        canonical_template_text(GRUND_TOML_TEMPLATE).replace("{NAME}", &escape_toml_basic(name));
    if let Some(description) = description {
        rendered = rendered.replace(
            "# project_description = \"<one line shown next to this project in workspace member lists>\"",
            &format!(
                "project_description = \"{}\"",
                escape_toml_basic(description)
            ),
        );
    }
    rendered
}
