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

/// §FS-init.2.3.4.3, §FS-init.2.3.4.9: the shared read and rename-maintenance guidance.
pub(super) const AGENTS_TEMPLATE: &str = include_str!("../../assets/templates/AGENTS.md");
/// Workspace-only project-boundary guidance (§FS-init.2.3.4.16).
pub(super) const AGENTS_PROJECT_NAMESPACES_TEMPLATE: &str =
    include_str!("../../assets/templates/AGENTS-project-namespaces.md");
/// The scaffold config carries one comment, its first line, saying where the
/// rest of the settings are; it declares no citation rule, and `grund guide
/// citations` prints the guide to them (§FS-init.2.4.5). The file lands verbatim
/// in the adopting repository, so it names no ID of this one: an ID would name a
/// document that reader does not have — or an unrelated one of their own
/// (§REQ-shipped-surfaces.1).
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
/// The `FS` starter where the governing config homes `FS` in one file, which
/// the template itself no longer does (§FS-init.2.1.3).
pub(crate) const FS_FILE_TEMPLATE: &str =
    include_str!("../../assets/templates/software-development/functional-spec-file.md");
/// The `FS` folder's index starter, the template's own home (§FS-init.2.1.3).
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
/// The core of the setup skill, its `SKILL.md`: what `grund guide setup` prints
/// (§FS-init.5.5). It opens with the fresh-repository path and the contents, then the
/// adoption workflow (§FS-init.5.1), and its validation step ends on the full-tree
/// scope (§FS-check.1.3).
///
/// Every file of the skill is printed byte-for-byte into whatever repository the agent
/// is standing in (§FS-init.5.3). That is why `skills/` is an unwalked home
/// (§FS-config.3.4.7.1) and why nothing in the skill cites an ID of this repository: it
/// would name a document the reader does not have (§REQ-shipped-surfaces.1), so its
/// links are the public URLs of the pages they used to cite. What it teaches is
/// grounded here instead, on the core and on each of [`AGENT_SETUP_SECTIONS`].
pub const AGENT_SETUP_INSTRUCTIONS: &str = include_str!("../../assets/skills/grund-init/SKILL.md");

/// One reference section of the setup skill: its name, and the text of
/// `references/<name>.md`, so the name and the file cannot disagree.
macro_rules! setup_section {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                "../../assets/skills/grund-init/references/",
                $name,
                ".md"
            )),
        )
    };
}

/// The name the core is printed by, the first of the guide's sections (§FS-init.5.5).
pub const SETUP_CORE_SECTION: &str = "setup";

/// What the front page says the core holds (§FS-init.5.5, §FS-init.5.6). A reference
/// section's summary is its Contents line instead, so it is written only there.
const SETUP_CORE_SUMMARY: &str = "the core: adopting grund in a repository that is not fresh, its workflow, questions and validation";

/// The setup skill's reference sections, the closed set in Contents order
/// (§FS-init.5.5). This table and [`SETUP_CORE_SECTION`] are the one list of the
/// guide's section names: the front page, the command's `-h` page, its
/// `known sections:` refusal and `--full` all read it (§AR-system.principle).
pub const AGENT_SETUP_SECTIONS: [(&str, &str); 10] = [
    // How to recommend each setting, then the config questions but `[citations]`.
    setup_section!("config"),
    // The canonical citation-directions page verbatim (§FS-config.3.9), held to it by
    // the asset-sync check.
    setup_section!("citations"),
    // Ends on the full-tree scope that reports what `[scan] include` left out
    // (§FS-check.1.3).
    setup_section!("languages"),
    setup_section!("members"),         // §FS-init.5.4
    setup_section!("scoped-guidance"), // §FS-init.2.3.5.11
    setup_section!("schema"),          // §FS-schema.14
    setup_section!("queries"),         // §FS-init.2.3.4.3.1
    setup_section!("editor"),          // §FS-lsp.2.3
    // The clickable integrations (§FS-integrations), the instruction files `--write`
    // synchronizes (§FS-integrations.4.3.8), and the committable opinion with its
    // per-agent gate (§DF-repo-conversation-opinion, §DF-conversation-link-target.2.4).
    setup_section!("integrations"),
    // The chapter-rules writing section of the rules guide verbatim (§FS-rules.10).
    setup_section!("rules"),
];

/// Which part of the setup skill a caller asks for (§FS-init.5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetupPart<'a> {
    /// One section by name: [`SETUP_CORE_SECTION`] is the core, any other name a
    /// reference section. `grund guide NAME`.
    Section(&'a str),
    /// The core, then every reference section in table order: `--full`.
    Full,
}

/// The text of one part of the setup skill, each file read through
/// [`canonical_template_text`], so `--full` is built from the same text each file
/// prints (§FS-init.5.3). `Full` joins the core and the sections with one newline
/// between consecutive files (§FS-init.5.5). `None` exactly when a `Section` names
/// no section, which the caller refuses with the guide's names (§FS-init.5).
pub fn setup_instructions(part: SetupPart<'_>) -> Option<String> {
    match part {
        SetupPart::Section(SETUP_CORE_SECTION) => {
            Some(canonical_template_text(AGENT_SETUP_INSTRUCTIONS))
        }
        SetupPart::Section(name) => AGENT_SETUP_SECTIONS
            .iter()
            .find(|(section, _)| *section == name)
            .map(|(_, text)| canonical_template_text(text)),
        SetupPart::Full => {
            let mut text = canonical_template_text(AGENT_SETUP_INSTRUCTIONS);
            for (_, section) in AGENT_SETUP_SECTIONS {
                text.push('\n');
                text.push_str(&canonical_template_text(section));
            }
            Some(text)
        }
    }
}

/// One line of the guide's front page: a section's name, what it holds, and the
/// bytes it prints (§FS-init.5.6).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuideSection {
    pub name: &'static str,
    pub summary: &'static str,
    pub bytes: usize,
}

/// Every section of the guide in `--full` order, the core first (§FS-init.5.5). A
/// reference section's summary is the phrase its Contents line ends with, without
/// the closing period, and empty if the core has no such line (§FS-init.5.6).
pub fn guide_sections() -> Vec<GuideSection> {
    let core = GuideSection {
        name: SETUP_CORE_SECTION,
        summary: SETUP_CORE_SUMMARY,
        bytes: canonical_template_text(AGENT_SETUP_INSTRUCTIONS).len(),
    };
    let references = AGENT_SETUP_SECTIONS.iter().map(|(name, text)| {
        let entry = format!("- `grund guide {name}` (`references/{name}.md`): ");
        let summary = AGENT_SETUP_INSTRUCTIONS
            .lines()
            .find_map(|line| line.strip_prefix(entry.as_str()))
            .map_or("", |phrase| phrase.trim_end().trim_end_matches('.'));
        GuideSection {
            name,
            summary,
            bytes: canonical_template_text(text).len(),
        }
    });
    std::iter::once(core).chain(references).collect()
}

/// The core's *Fresh repository* section, from its heading up to the core's next
/// `##` heading, which the front page prints byte for byte (§FS-init.5.6). Empty if
/// the core has no such section.
pub fn setup_fresh_path() -> String {
    let core = canonical_template_text(AGENT_SETUP_INSTRUCTIONS);
    let Some(start) = core.find("\n## Fresh repository\n").map(|at| at + 1) else {
        return String::new();
    };
    let end = core[start + 1..]
        .find("\n## ")
        .map_or(core.len(), |at| start + 1 + at + 1);
    core[start..end].to_string()
}

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

/// The generated `grund.toml` — `template`'s minimal version-2 config, with
/// only `project_name` substituted (§FS-init.2.4.3, §FS-init.2.4.9). With
/// `--description`, a `[presentation]` table holding `description` stands right
/// above the first `[presentation.kinds]` table, or ends a file that has none
/// (§FS-init.2.4.7, §DF-workspace-member-descriptions).
pub(crate) fn render_grund_toml(
    template: Template,
    name: &str,
    description: Option<&str>,
) -> String {
    let mut rendered =
        canonical_template_text(template.grund_toml()).replace("{NAME}", &escape_toml_basic(name));
    if let Some(description) = description {
        let table = format!(
            "[presentation]\ndescription = \"{}\"\n",
            escape_toml_basic(description)
        );
        match rendered.find("\n[presentation.kinds.") {
            Some(at) => rendered.insert_str(at + 1, &table),
            None => {
                if !rendered.ends_with('\n') {
                    rendered.push('\n');
                }
                rendered.push('\n');
                rendered.push_str(&table);
            }
        }
    }
    rendered
}
