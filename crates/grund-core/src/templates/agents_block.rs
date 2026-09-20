//! The managed `AGENTS.md` block (§FS-init.2.3): every placeholder of
//! `templates/AGENTS.md` substituted from the effective config, and the H1
//! scaffolding line a fresh repository's file carries above it.
//!
//! What a managed block *should say*, as a function of config alone
//! (§AR-system.2.11). That is what lets `init` write the block and `check`
//! re-render two of its sections and byte-compare them for drift
//! (§FS-check.3.5): rendering is deterministic, so a fresh render is the hash.
//! Nothing here reads the tree — the one section that does, the workspace
//! members of §FS-init.2.3.4.15, arrives already rendered from the run that
//! walked for it (`writers/init_render.rs`).

use super::assets::{AGENTS_TEMPLATE, canonical_template_text};
use super::citation_directions::citation_directions_section;
use super::clickable_citations::{ConversationSurface, clickable_citations_section};
use crate::config::{Config, kind_prefixes};
use crate::grammar::{id_shape, inline_note_layout_sentence};
use crate::model::plural;

/// The substitutions that turn `templates/AGENTS.md` into a concrete `AGENTS.md`
/// for a repo (§FS-init.2.3.8): the project name, plus the ID/marker shape taken
/// from the config `grund init` leaves in place — so a `{kind}-{slug}` repo gets a
/// `<KIND>-<slug>` description, a strict repo gets the strict-mode note, custom
/// kinds show up in the kind set, and so on. Everything *not* substituted here is
/// fixed for the block version. `{ID_SHAPE_SEC}` is listed before `{ID_SHAPE}`
/// only for readability; neither placeholder is a substring of the other.
/// `workspace_members` arrives already rendered (§FS-init.2.3.4.15): it is the
/// one substitution that reads the tree rather than the config, it does not vary
/// by surface, and its walk-up must run once per `init` invocation rather than
/// once per entrypoint file — see `writers/init_render.rs`. Which is also why
/// nothing here needs the target directory any more.
///
/// Why the worked citation example is escaped: a live marker would make the
/// generated block fail the host repo's own `grund check` as a dangling
/// reference.
fn agents_template_substitutions(
    name: &str,
    config: &Config,
    workspace_members: &str,
    surface: ConversationSurface,
) -> Vec<(&'static str, String)> {
    let sep = config.section_separator.as_str();
    let marker = config.marker.as_str();
    let id_shape = id_shape(&config.id_format);
    let id_example = config
        .id_format
        .replace("{kind}", "FS")
        .replace("{number}", "042")
        .replace("{slug}", "user-login");
    // §FS-init.2.3.8.2: the worked example illustrates a non-existent ID, so it is
    // rendered in the `<marker>`-escaped form (§FS-workspace.1).
    let cite_example = format!("<{marker}>{id_example}{sep}3{sep}1");
    let kinds_set = format!("{{{}}}", kind_prefixes(&config.kinds).join(", "));
    let bare_note = if config.strict {
        format!(
            "Bare ID-shaped tokens are ignored — `[reference] strict = true` is set in `grund.toml`, so only `{marker}`-prefixed citations are checked."
        )
    } else {
        format!(
            "Bare ID-shaped tokens are also recognized as citations because `[reference] strict = false` is set in `grund.toml`; remove that compatibility override or set strict back to `true` to require the `{marker}` marker (run `grund fmt --marker` first to upgrade existing bare citations)."
        )
    };
    let section_heading_note = section_heading_note(config, marker);
    let inline_citation_style = inline_citation_style_sentence(config);
    vec![
        ("{NAME}", name.to_string()),
        ("{ID_SHAPE_SEC}", format!("{id_shape}[{sep}<section>]")),
        ("{ID_SHAPE}", id_shape),
        ("{ID_EXAMPLE}", id_example),
        ("{CITE_EXAMPLE}", cite_example),
        ("{KINDS_SET}", kinds_set),
        ("{BARE_TOKEN_NOTE}", bare_note),
        ("{SECTION_HEADING_NOTE}", section_heading_note),
        ("{INLINE_CITATION_STYLE}", inline_citation_style),
        ("{MARKER}", marker.to_string()),
        ("{TRIGGER}", config.trigger.clone()),
        ("{DECLARATION_MAP}", declaration_map(config)),
        ("{CITATION_DIRECTIONS}", citation_directions_section(config)),
        (
            "{CLICKABLE_CITATIONS}",
            clickable_citations_section(config, surface),
        ),
        ("{WORKSPACE_MEMBERS}", workspace_members.to_string()),
    ]
}

fn section_heading_note(config: &Config, marker: &str) -> String {
    let sep = config.section_separator.as_str();
    let unmarked_policy = " Every non-declaration heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, headings that close the body, fenced examples, non-ATX text, and source doc-comments stay exempt, while bold labels remain the non-citable alternative.";
    // §FS-init.2.3.4.5: enabled repositories teach explicit complete handles;
    // the generated false default retains the prior numeric-only bytes here.
    if config.named_sections {
        let verdict = match config.section_heading_levels.as_str() {
            "strict" => "an error",
            "warn" => "a warning",
            _ => "recommended for readability",
        };
        return format!(
            "Named sections are enabled: use explicit complete paths (`## goals: Goals`, `### goals.performance: Performance`, `### goals.3: Ordered child`) so `{marker}<ID>{sep}goals.performance` resolves; handles are letter-first lowercase names, `name.number` is legal, and `number.name` is reserved. Heading depth must match each path component ({verdict}). Purely numbered headings remain citable.{unmarked_policy}"
        );
    }
    match config.section_heading_levels.as_str() {
        "strict" => format!(
            "Numbered headings inside a declaration are citable sections: use depth-matching headings (`## 1. …`, `### 1.1 …`, etc.) so `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve; mismatched heading depth is a `grund check` error.{unmarked_policy}"
        ),
        "warn" => format!(
            "Numbered headings inside a declaration are citable sections: use depth-matching headings (`## 1. …`, `### 1.1 …`, etc.) so `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve; mismatched heading depth is a `grund check` warning.{unmarked_policy}"
        ),
        _ => format!(
            "Numbered headings inside a declaration are citable sections: `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve by dotted number, and depth-matching headings (`## 1. …`, `### 1.1 …`) are recommended for readability.{unmarked_policy}"
        ),
    }
}

/// §FS-inline-citation-style.5.4: the sentence that closes the rendered copy at
/// every `inline_style`, after whatever the other keys produced, so the author
/// and the linter agree on where the shape rules stop
/// (§FS-inline-citation-style.1.1). It moves no managed-block version: it only
/// widens what an author may write, so a block that predates it teaches a
/// narrower rule than the gate enforces — an over-careful comment, never a
/// finding.
const DOC_COMMENT_SENTENCE: &str = " Doc-comments (`///`, `//!`, `/** */`, a docstring, a comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.";

/// §FS-inline-citation-style.5.2: the sentence that follows the budgets and
/// precedes the layout sentence, under `citation-with-note` only — restating
/// §FS-inline-citation-style.1's block rule at the point an agent needs it to act on a cap finding. It
/// moves no managed-block version, for the same reason the layout and
/// doc-comment sentences do not (§FS-inline-citation-style.2.2): a block that predates it teaches the
/// same rule less precisely, an over-careful comment, never a finding.
const BLOCK_SENTENCE: &str =
    " A note is one comment block: a blank line splits it, an empty comment line does not.";

pub(crate) fn inline_citation_style_sentence(config: &Config) -> String {
    if config.inline_style == "citation-only" {
        return format!(
            "Inline citations carry no prose — put rationale in the spec.{DOC_COMMENT_SENTENCE}"
        );
    }
    let budgets = if config.inline_note_suggested_lines == config.inline_note_max_lines {
        format!(
            "Inline notes: ≤ {} line{}, ≤ {} columns.",
            config.inline_note_max_lines,
            plural(config.inline_note_max_lines),
            config.inline_note_max_columns
        )
    } else {
        format!(
            "Inline notes: ≤ {} line{} preferred, hard cap {} lines; ≤ {} columns.",
            config.inline_note_suggested_lines,
            plural(config.inline_note_suggested_lines),
            config.inline_note_max_lines,
            config.inline_note_max_columns
        )
    };
    // §FS-inline-citation-style.5.3: the layout sentence appends to the budgets
    // and the block sentence, empty under `any`, so a project with no layout
    // renders the byte-identical block it rendered before that key existed.
    format!(
        "{budgets}{BLOCK_SENTENCE}{}{DOC_COMMENT_SENTENCE}",
        inline_note_layout_sentence(config.lexical())
    )
}

fn markdown_link_label(raw: &str) -> String {
    raw.replace('\\', r"\\")
        .replace('[', r"\[")
        .replace(']', r"\]")
}

pub(crate) fn markdown_link_destination(raw: &str) -> String {
    if raw
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '(' | ')' | '<' | '>'))
    {
        format!("<{}>", raw.replace('\\', r"\\").replace('>', r"\>"))
    } else {
        raw.to_string()
    }
}

/// The Project map rows (§FS-init.2.3.4.4): one per configured kind, linking
/// its home. A **citable** kind is named by its kind name — the prefix an agent
/// will type in a citation. A **non-citable** kind is named by its *place*: it
/// has no ID namespace, so its name is a config handle and the only useful thing
/// to show an agent is the directory to go and read.
///
/// Either way, every kind with a home links to it, and an unwalked kind is one
/// of them: its row is why it is configured.
fn declaration_map(config: &Config) -> String {
    // §FS-init.2.3.4.4.1: the homeless kind gets no row. Every row is a link to a
    // place, and it is the one kind that is not a place — the complement of all
    // of them. Its citation directions still render (§FS-init.2.3.5).
    let homeless = config.homeless_kind();
    let rows = config
        .kinds
        .iter()
        .filter(|kind| kind.kind != homeless)
        .map(|kind| {
            let title = kind.title.as_deref().unwrap_or("Declaration");
            // A non-citable kind is labelled by its home, which `place_label`
            // already renders. An unwalked kind (§FS-config.3.4.7) is one of them;
            // its missing directions bullet is §FS-init.2.3.5's.
            match (
                kind.file.as_deref().or(kind.folder.as_deref()),
                kind.citable,
            ) {
                (Some(home), true) => row(&kind.kind, home, title),
                (Some(home), false) => row(&kind.place_label().unwrap_or_default(), home, title),
                (None, _) => format!(
                    "- `{}`: {title} (inline / configured by convention)",
                    kind.kind.replace('`', "\\`")
                ),
            }
        });
    rows.collect::<Vec<_>>().join("\n")
}

/// One Project map row: a Markdown link from `label` to `home`, then the title.
fn row(label: &str, home: &str, title: &str) -> String {
    format!(
        "- [{}]({}): {title}",
        markdown_link_label(label),
        markdown_link_destination(home)
    )
}

/// The managed block — just the H2 section that `init` appends to, or replaces
/// inside, an existing `AGENTS.md` (§FS-init.2.3.10). The template *is* the block;
/// the H2 line carrying the version is its own begin marker (§FS-init.2.3.1).
/// `workspace_members` is the §FS-init.2.3.4.15 section, rendered once per `init`
/// invocation by `writers/init_render.rs` against the directory being
/// initialized and handed to every surface that block is written to.
pub(crate) fn render_agents_append_block(
    name: &str,
    config: &Config,
    workspace_members: &str,
    surface: ConversationSurface,
) -> String {
    let mut rendered = canonical_template_text(AGENTS_TEMPLATE);
    for (placeholder, value) in
        agents_template_substitutions(name, config, workspace_members, surface)
    {
        rendered = rendered.replace(placeholder, &value);
    }
    rendered
}

/// The full generated `AGENTS.md` for a fresh repo from a pre-rendered managed
/// block (§FS-init.2.3.10) — the H1 scaffolding line, which is *unmanaged*, then
/// the block. Taking the block rather than rendering it is what lets one `init`
/// run reuse it as both the full body *and* the append/update payload, so the
/// workspace-members walk-up (§FS-init.2.3.4.15) runs once per invocation.
pub(crate) fn render_agents_md_from_block(name: &str, block: &str) -> String {
    format!("# {name} — agent instructions\n\n{block}")
}
