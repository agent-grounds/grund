//! The templates component (§AR-system.2.11): what a managed block *should
//! say*, as a function of the effective config and nothing else — the
//! `AGENTS.md` block of §FS-init.2.3 with its two generated sections, and the
//! `grund.toml` and scaffold payload `grund init` writes beside it
//! (§FS-init.2.1, §FS-init.2.4). It reads the config's records — `&Project`,
//! and `&Compiled` where a sentence needs the lexical settings, never the
//! `Config` façade (§AR-config.5) — and the grammar's managed-block markers,
//! knows no file, no rule and no report, and writes nothing.
//!
//! It is a component rather than a corner of the writers because two commands
//! need the same answer from opposite directions: `init` writes the block, and
//! `check` re-renders two of its sections and byte-compares them for drift
//! (§FS-check.3.5, §AR-checker.2.7). Rendering is deterministic, so a fresh
//! render is the hash — and while the renderers sat in `writers/`, the checker
//! read three of them upward out of a component above it (§AR-system.4).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). Four files, out of the writers' former
//! `init_templates.rs` and `init_citation_directions.rs`: the embedded payload
//! and its newline canonicalization, the block substitutions, the
//! `### Citation directions` renderer, and the `### Clickable citations`
//! renderer with the entrypoint surface it varies by (§FS-init.2.3.4.17).
//!
//! What it does **not** hold is anything a run decides. Which entrypoint files
//! a repository has is a probe over the tree and is
//! `scanner/agent_entrypoints.rs`; which of them this invocation writes is
//! `writers/init_plan.rs`; the effective config a run leaves governing the
//! target and workspace discovery (§FS-init.2.3.4.16) are `writers/init_render.rs`,
//! which hands presence and the rendered members (§FS-init.2.3.4.15) in together.

mod agents_block;
mod assets;
mod citation_directions;
mod clickable_citations;
mod scoped_guidance;

pub use assets::{
    AGENT_SETUP_INSTRUCTIONS, AGENT_SETUP_SECTIONS, GuideSection, SETUP_CORE_SECTION, SetupPart,
    canonical_template_text, guide_sections, setup_fresh_path, setup_instructions,
};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use agents_block::{
    WorkspaceGuidance, markdown_link_destination, render_agents_append_block,
    render_agents_md_from_block,
};
pub(crate) use assets::{
    AS_README_TEMPLATE, DA_README_TEMPLATE, DF_README_TEMPLATE, E2E_README_TEMPLATE,
    FS_FILE_TEMPLATE, FS_README_TEMPLATE, GITKEEP_TEMPLATE, GOALS_TEMPLATE, GRUND_DOC_TEMPLATE,
    REQ_README_TEMPLATE, Template, render_grund_toml, render_scaffold_id_shapes,
};
pub(crate) use citation_directions::citation_directions_section;
pub(crate) use clickable_citations::{ConversationSurface, clickable_citations_section};
pub(crate) use scoped_guidance::{Layout, declaration_shapes_section};

// What another component's tests read (§AR-core-module-layout.1.3): the
// inline-citation-style sentence the checker's layout cases read per key. The
// citation-level legend went beside the cases that assert it stands alone.
#[cfg(test)]
pub(crate) use agents_block::inline_citation_style_sentence;
#[cfg(test)]
pub(crate) use agents_block::render_project_namespaces_section;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_citation_directions;
#[cfg(test)]
mod tests_citation_grounding;
#[cfg(test)]
mod tests_clickable_citations;
#[cfg(test)]
mod tests_scoped_guidance;
#[cfg(test)]
mod tests_setup_instructions;
