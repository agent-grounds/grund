//! The scaffold payload `grund init` writes (§FS-init.2.1): the reference
//! templates, embedded in the binary from `templates/` in the source tree, the
//! newline canonicalization every one of them is read through, and the
//! `grund.toml` render that is nothing but a name substituted into one of them
//! (§FS-init.2.4).
//!
//! Data rather than a decision, which is why it is here and not in the run:
//! `writers/init_template.rs` decides *which* of these a run writes and where,
//! and this file only says what each template holds (§AR-system.2.11). Each
//! template is a directory of its own under `assets/templates/`, and the
//! managed block's `AGENTS.md`, which belongs to none, sits beside them
//! (§DF-init-templates.2.5).

use crate::config::{Project, escape_toml_basic};
use crate::grammar::id_shape;

pub(super) const AGENTS_TEMPLATE: &str = include_str!("../../assets/templates/AGENTS.md");
/// The scaffold config teaches open-ended marker choices (§FS-init.2.4.5).
/// Its `[citations]` block comment explains the five levels
/// and hands the reader the `CITATION_DIRECTIONS_URL` of
/// `citation_directions.rs`, where it used to cite
/// §FS-config.3.9 by ID: the file lands verbatim in the adopting repository, so
/// an ID of this one names a document that reader does not have — or an
/// unrelated one of their own (§REQ-shipped-surfaces.1).
const GRUND_TOML_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/grund.toml");
/// The `none` template's config: the default one with `kinds = []` for its
/// kind tables, and nothing else (§FS-init.2.4.9). It keeps the same marker
/// guidance (§FS-init.2.4.5).
const NO_KINDS_GRUND_TOML_TEMPLATE: &str = include_str!("../../assets/templates/none/grund.toml");

/// The templates built into the binary, which `grund init --template` names
/// (§FS-init.1.6), in the order a refusal lists them (§FS-init.1.6.3).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Template {
    /// The config with the canonical kinds, and its starter files
    /// (§FS-init.1.6.1). What `init` lays when no template is named.
    #[default]
    SoftwareDevelopment,
    /// A config that declares no kinds, and no starter files (§FS-init.1.6.2).
    None,
}

impl Template {
    /// Every built-in template, in the order a refusal names them.
    pub(crate) const ALL: [Template; 2] = [Template::SoftwareDevelopment, Template::None];

    /// The name `--template` takes.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Template::SoftwareDevelopment => "software-development",
            Template::None => "none",
        }
    }

    /// The built-in template `name` names, if any (§FS-init.1.6.3).
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|template| template.name() == name)
    }

    /// Whether the template has starter files for `--scaffold` to lay
    /// (§FS-init.1.6.4).
    pub(crate) fn has_starter_files(self) -> bool {
        self == Template::SoftwareDevelopment
    }

    fn grund_toml(self) -> &'static str {
        match self {
            Template::SoftwareDevelopment => GRUND_TOML_TEMPLATE,
            Template::None => NO_KINDS_GRUND_TOML_TEMPLATE,
        }
    }
}
pub(crate) const GRUND_DOC_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/grund.md");
pub(crate) const GOALS_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/goals.md");
pub(crate) const REQUIREMENTS_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/requirements.md");
pub(crate) const FS_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/functional-spec-README.md");
/// The `REQ` folder's index starter (§FS-init.2.1.3, §FS-init.2.4.10).
pub(crate) const REQ_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/requirements-README.md");
pub(crate) const E2E_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/e2e-README.md");
pub(crate) const AS_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/architecture-README.md");
pub(crate) const DF_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/decisions-functional-README.md");
pub(crate) const DA_README_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/decisions-architectural-README.md");
pub(crate) const GITKEEP_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/gitkeep.md");
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

/// Render one docs-scaffold template's owned ID examples from the illustrated
/// kind's effective grammar. An absent kind row and a row without an override
/// both use the repository format. Bind the kind placeholder before rendering
/// the remaining schematic components so configured literals stay unchanged
/// (§FS-init.2.1.3).
pub(crate) fn render_scaffold_id_shapes(template: &str, kind: &str, project: &Project) -> String {
    let effective_format = project
        .schema
        .rows
        .iter()
        .find(|row| row.name == kind)
        .and_then(|row| row.kind.as_ref()?.id_format.as_deref())
        .unwrap_or(&project.schema.ids.format);
    let shape = id_shape(&effective_format.replace("{kind}", kind));
    canonical_template_text(template).replace(&format!("{{{kind}_ID_SHAPE}}"), &shape)
}

/// The generated `grund.toml` — `template`'s config, every default written out
/// explicitly as a teaching surface, with only `project_name` substituted
/// (§FS-init.2.4.3, §FS-init.2.4.9). With `--description`, the commented
/// `project_description` teaching line becomes the real key (§FS-init.2.4.7,
/// §DF-workspace-member-descriptions).
pub(crate) fn render_grund_toml(
    template: Template,
    name: &str,
    description: Option<&str>,
) -> String {
    let mut rendered =
        canonical_template_text(template.grund_toml()).replace("{NAME}", &escape_toml_basic(name));
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
