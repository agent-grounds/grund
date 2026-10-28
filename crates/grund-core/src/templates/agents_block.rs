//! The managed `AGENTS.md` block (§FS-init.2.3): every placeholder of
//! `templates/AGENTS.md` substituted from the effective config, and the H1
//! scaffolding line a fresh repository's file carries above it.
//!
//! What a managed block *should say*, as a function of config alone
//! (§AR-system.2.11). That is what lets `init` write the block and the writers
//! render the whole block into `Expected`, which `check` byte-compares for
//! drift (§FS-check.3.5.4): rendering is deterministic, so a fresh render is
//! the hash.
//! Nothing here reads the tree — workspace presence (§FS-init.2.3.4.16) and
//! member text (§FS-init.2.3.4.15) arrive from the run that walked for them
//! (`writers/init_render.rs`).

use super::assets::{AGENTS_PROJECT_NAMESPACES_TEMPLATE, AGENTS_TEMPLATE, canonical_template_text};
use super::citation_directions::{citation_directions_placeholder, citation_directions_section};
use super::clickable_citations::{ConversationSurface, clickable_citations_section};
use crate::config::{Compiled, KindConfig, Place, Project, Run, guided_kind, kind_prefixes};
use crate::grammar::{
    AGENTS_BLOCK_BASE_VERSION, agents_block_heading, id_shape, inline_note_layout_sentence,
    literal_after_kind_placeholder,
};
use crate::model::plural;

/// Workspace discovery and member-list expansion are independent (§FS-init.2.3.4.16).
/// A discovered workspace retains project-boundary guidance even when its members
/// cannot be rendered (§FS-init.2.3.4.15). Writers settle both once per invocation.
#[derive(Default)]
pub(crate) struct WorkspaceGuidance {
    pub present: bool,
    pub members: String,
}

/// The fixed workspace-only fragment, with the configured marker (§FS-init.2.3.4.16).
pub(crate) fn render_project_namespaces_section(marker: &str) -> String {
    format!(
        "\n\n{}",
        canonical_template_text(AGENTS_PROJECT_NAMESPACES_TEMPLATE)
            .trim_end_matches('\n')
            .replace("{MARKER}", marker)
    )
}

/// The substitutions that turn `templates/AGENTS.md` into a concrete `AGENTS.md`
/// for a repo (§FS-init.2.3.8): the project name, plus the ID/marker shape taken
/// from the config `grund init` leaves in place — so a `{kind}-{slug}` repo gets a
/// `<KIND>-<slug>` description, a strict repo gets the strict-mode note, custom
/// kinds show up in the kind set, and so on. Everything *not* substituted here is
/// fixed for the block version. `{ID_SHAPE_SEC}` is listed before `{ID_SHAPE}`
/// only for readability; neither placeholder is a substring of the other.
/// `workspace` arrives already discovered (§FS-init.2.3.4.16), with its members
/// rendered (§FS-init.2.3.4.15): these read the tree rather than the config and do not vary
/// by surface, and its walk-up must run once per `init` invocation rather than
/// once per entrypoint file — see `writers/init_render.rs`. Which is also why
/// nothing here needs the target directory any more.
///
/// Why the worked citation example is escaped: a live marker would make the
/// generated block fail the host repo's own `grund check` as a dangling
/// reference.
fn agents_template_substitutions(
    name: &str,
    (project, run, compiled): (&Project, &Run, &Compiled),
    workspace: &WorkspaceGuidance,
    surface: ConversationSurface,
) -> Vec<(&'static str, String)> {
    let ids = &project.schema.ids;
    let sep = ids.section_separator.as_str();
    let marker = project.schema.citation.marker.as_str();
    let id_shape = id_shape(&ids.format);
    let id_example = ids
        .format
        .replace("{kind}", "FS")
        .replace("{number}", "042")
        .replace("{slug}", "user-login");
    // §FS-init.2.3.8.2: the worked example illustrates a non-existent ID, so it is
    // rendered in the `<marker>`-escaped form (§FS-workspace.1).
    let cite_example = format!("<{marker}>{id_example}{sep}3{sep}1");
    // §FS-init.2.3.4.4.2: a config with no citable kind says so rather than
    // naming the empty set.
    let kind_configs = project.kind_configs();
    let citable = kind_prefixes(&kind_configs);
    let kinds_set = if citable.is_empty() {
        "no `KIND` is declared yet".to_string()
    } else {
        format!("`KIND ∈ {{{}}}`", citable.join(", "))
    };
    // §FS-init.2.3.8.1: v2 fixes marked-only recognition; only v1 has a strict switch.
    let bare_note = if project.version == 2 {
        format!(
            "Bare ID-shaped tokens are ignored — config version 2 recognizes only `{marker}`-prefixed citations, with no opt-out."
        )
    } else if project.schema.citation.strict {
        format!(
            "Bare ID-shaped tokens are ignored — `[reference] strict = true` is set in `grund.toml`, so only `{marker}`-prefixed citations are checked."
        )
    } else {
        format!(
            "Bare ID-shaped tokens are also recognized as citations because `[reference] strict = false` is set in `grund.toml`; remove that compatibility override or set strict back to `true` to require the `{marker}` marker (run `grund fmt --marker` first to upgrade existing bare citations)."
        )
    };
    let section_heading_note = section_heading_note(project, marker);
    let inline_citation_style = inline_citation_style_sentence(project, compiled);
    // §FS-init.2.3.4.3.1: `<KIND>`, the literal the format writes after `{kind}`, then `*`.
    let glob_pattern = format!(
        "<KIND>{}*",
        literal_after_kind_placeholder(&ids.format).unwrap_or("")
    );
    vec![
        // §FS-init.2.3.9.3: the base revision; `Guidance` moves it to the run's.
        (
            "{BLOCK_HEADING}",
            agents_block_heading(AGENTS_BLOCK_BASE_VERSION),
        ),
        ("{NAME}", name.to_string()),
        ("{GLOB_PATTERN}", glob_pattern),
        ("{ID_SHAPE_SEC}", format!("{id_shape}[{sep}<section>]")),
        ("{ID_SHAPE}", id_shape),
        ("{CITE_EXAMPLE}", cite_example),
        ("{KINDS_SET}", kinds_set),
        ("{KIND_DISCOVERY}", kind_discovery_bullet(&kind_configs)),
        ("{BARE_TOKEN_NOTE}", bare_note),
        ("{SECTION_HEADING_NOTE}", section_heading_note),
        ("{INLINE_CITATION_STYLE}", inline_citation_style),
        ("{MARKER}", marker.to_string()),
        ("{TRIGGER}", project.presentation.trigger.clone()),
        ("{DECLARATION_MAP}", declaration_map(project)),
        (
            "{CITATION_DIRECTIONS}",
            citation_directions_placeholder(citation_directions_section(project, run)),
        ),
        (
            "{CLICKABLE_CITATIONS}",
            clickable_citations_section(project, surface),
        ),
        (
            "{PROJECT_NAMESPACES}",
            if workspace.present {
                render_project_namespaces_section(marker)
            } else {
                String::new()
            },
        ),
        ("{WORKSPACE_MEMBERS}", workspace.members.clone()),
    ]
}

/// The discovery rung (§FS-init.2.3.4.3): its `--kind` example names the kind
/// init's guidance names, then the first other citable kind with a `folder`
/// home, in config order, so the recipe runs in the project it was rendered for;
/// a config with no citable kind gets the bare `grund list` alone.
fn kind_discovery_bullet(kinds: &[KindConfig]) -> String {
    let Some(first) = guided_kind(kinds) else {
        return "- `grund list` — discover IDs if you get lost".to_string();
    };
    let second = kinds
        .iter()
        .find(|row| row.citable && row.folder.is_some() && row.kind != first.kind);
    let example = match second {
        Some(second) => format!("{},{}", first.kind, second.kind),
        None => first.kind.clone(),
    };
    format!("- `grund list` / `grund list --kind {example}` — discover IDs if you get lost")
}

fn section_heading_note(project: &Project, marker: &str) -> String {
    let ids = &project.schema.ids;
    let sep = ids.section_separator.as_str();
    // §FS-init.2.3.4.5: since v16 the relative form, and the lock a citation of it needs;
    // since v23 in a doc-comment declaration body too.
    let (parent, child, path) = if ids.named_sections {
        ("## format: Formatting", "### .1: Username", "format.1")
    } else {
        ("## 3. Third", "### .1. Scope", "3.1")
    };
    let heading_policy = format!(
        " In a declaration body, Markdown or doc-comment, a heading may instead write only the component it adds and take the rest from the heading one level up: `{child}` under `{parent}` declares `{marker}<ID>{sep}{path}`, the one coordinate it is cited and reported by. After citing such a heading, run `grund fmt --write` so the section lock records it, and commit the lock. Every non-declaration heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, headings that close the body, fenced examples, non-ATX text, and source doc-comments stay exempt, while bold labels remain the non-citable alternative."
    );
    // §FS-init.2.3.4.5: enabled repositories teach explicit complete handles, and since
    // v17 a name under a number, in a doc-comment too since v23, which the section lock guards.
    if ids.named_sections {
        let verdict = match ids.section_heading_levels.as_str() {
            "strict" => "an error",
            "warn" => "a warning",
            // §FS-config-v2.schema.1: `heading_depth = "should"`.
            "suggest" => "a suggestion",
            _ => "recommended for readability",
        };
        return format!(
            "Named sections are enabled: use explicit complete paths (`## goals: Goals`, `### goals.performance: Performance`, `### goals.3: Ordered child`) so `{marker}<ID>{sep}goals.performance` resolves; handles are letter-first lowercase names, and `name.number` and `number.name` are both legal; a cited address with a name under a number is recorded in the section lock. Heading depth must match each path component ({verdict}). Purely numbered headings remain citable.{heading_policy}"
        );
    }
    match ids.section_heading_levels.as_str() {
        "strict" => format!(
            "Numbered headings inside a declaration are citable sections: use depth-matching headings (`## 1. …`, `### 1.1 …`, etc.) so `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve; mismatched heading depth is a `grund check` error.{heading_policy}"
        ),
        "warn" => format!(
            "Numbered headings inside a declaration are citable sections: use depth-matching headings (`## 1. …`, `### 1.1 …`, etc.) so `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve; mismatched heading depth is a `grund check` warning.{heading_policy}"
        ),
        _ => format!(
            "Numbered headings inside a declaration are citable sections: `{marker}<ID>{sep}1` / `{marker}<ID>{sep}1.1` resolve by dotted number, and depth-matching headings (`## 1. …`, `### 1.1 …`) are recommended for readability.{heading_policy}"
        ),
    }
}

/// §FS-inline-citation-style.5.4: the sentence that closes the rendered copy at
/// every `inline_style`, after whatever the other keys produced, so the author
/// and the linter agree on where the shape rules stop
/// (§FS-inline-citation-style.1.1). Its bytes are §FS-inline-citation-style.5.4's
/// verbatim, and the position languages it names are
/// §FS-inline-citation-style.1.1.2's: a plain `//` or `#` block above a
/// definition is an inline note in every other language, so naming them all
/// taught a rule the gate does not enforce. Like every rendered line it is
/// compared by `check` (§FS-inline-citation-style.5.5), so a change to its wording
/// moves the block version (§FS-init.2.3.7).
const DOC_COMMENT_SENTENCE: &str = " Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell, SQL or Lisp comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.";

/// §FS-inline-citation-style.5.2: the sentence that follows the budgets and
/// precedes the layout sentence, under `citation-with-note` only — restating
/// §FS-inline-citation-style.1's block rule at the point an agent needs it to act on a cap finding
/// (§FS-inline-citation-style.2.2). `check` compares it as it does the doc-comment sentence
/// (§FS-inline-citation-style.5.5), so a change to its wording moves the block version
/// (§FS-init.2.3.7).
const BLOCK_SENTENCE: &str =
    " A note is one comment block: a blank line splits it, an empty comment line does not.";

pub(crate) fn inline_citation_style_sentence(project: &Project, compiled: &Compiled) -> String {
    let notes = &project.schema.notes;
    if notes.inline_style == "citation-only" {
        return format!(
            "Inline citations carry no prose — put rationale in the spec.{DOC_COMMENT_SENTENCE}"
        );
    }
    let budgets = if notes.suggested_lines == notes.max_lines {
        format!(
            "Inline notes: ≤ {} line{}, ≤ {} columns.",
            notes.max_lines,
            plural(notes.max_lines),
            notes.max_columns
        )
    } else {
        format!(
            "Inline notes: ≤ {} line{} preferred, hard cap {} lines; ≤ {} columns.",
            notes.suggested_lines,
            plural(notes.suggested_lines),
            notes.max_lines,
            notes.max_columns
        )
    };
    // §FS-inline-citation-style.5.3: the layout sentence appends to the budgets
    // and the block sentence, empty under `any`, so a project with no layout
    // renders the byte-identical block it rendered before that key existed.
    format!(
        "{budgets}{BLOCK_SENTENCE}{}{DOC_COMMENT_SENTENCE}",
        inline_note_layout_sentence(compiled.lexical(&project.schema))
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

/// The title of an untitled complement row in the Project map
/// (§FS-init.2.3.4.4.3): rendered wording, never a config default.
const COMPLEMENT_ROW_TITLE: &str = "Every scanned file no other row claims";

/// The Project map rows (§FS-init.2.3.4.4): one per configured kind, linking
/// its home. A **citable** kind is named by its kind name — the prefix an agent
/// will type in a citation. A **non-citable** kind is named by its *place*: it
/// has no ID namespace, so its name is a config handle and the only useful thing
/// to show an agent is the directory to go and read.
///
/// Either way, every kind with a home links to it, and an unwalked kind is one
/// of them: its row is why it is configured.
fn declaration_map(project: &Project) -> String {
    // The homeless kind gets no row unless a v2 `code` row lists places
    // (§FS-init.2.3.4.4.1, §FS-config-v2.schema.places.complement); its
    // citation directions still render (§FS-init.2.3.5).
    let homeless = project.schema.complement_name();
    let rows = project.schema.rows.iter().filter_map(|row| {
        // An untitled complement row says it is the complement (§FS-init.2.3.4.4.3).
        let untitled = if row.name == homeless {
            COMPLEMENT_ROW_TITLE
        } else {
            "Declaration"
        };
        let title = project
            .presentation
            .kinds
            .get(&row.name)
            .map_or(untitled, String::as_str);
        let places: Vec<&Place> = row
            .places
            .iter()
            .filter(|place| place.path().is_some())
            .collect();
        if places.is_empty() {
            // A citable kind with no home is declared inline only.
            return (row.name != homeless && row.kind.is_some()).then(|| {
                format!(
                    "- `{}`: {title} (inline / configured by convention)",
                    row.name.replace('`', "\\`")
                )
            });
        }
        // A citable kind is named by its name, a non-citable or unwalked one
        // (§FS-config.3.4.7, §FS-init.2.3.5) by its place; each further place is
        // one more link, in place order (§FS-init.2.3.4.4.1).
        let links = places.iter().enumerate().map(|(index, place)| {
            let home = place.path().unwrap_or_default();
            let label = match (&row.kind, index) {
                (Some(_), 0) => row.name.clone(),
                (Some(_), _) => home.to_string(),
                (None, _) => place.label().unwrap_or_default(),
            };
            link(&label, home)
        });
        Some(format!(
            "- {}: {title}",
            links.collect::<Vec<_>>().join(", ")
        ))
    });
    let rows = rows.collect::<Vec<_>>();
    if rows.is_empty() {
        return first_kind_paragraph(project);
    }
    rows.join("\n")
}

/// §FS-init.2.3.4.4.2: the Project map of a config with no row to list — one
/// paragraph teaching the first kind in the config's own spelling, rather than
/// an empty map.
fn first_kind_paragraph(project: &Project) -> String {
    if project.version == 2 {
        return "No kinds are declared yet. Add one `[schema.kinds.<KIND>]` table per kind of document this repository gives IDs to, with the `folders` or `files` its declarations live in, and run `grund init` again to list them here.".to_string();
    }
    "No kinds are declared yet: `grund.toml` sets `kinds = []`. Replace it with one `[[kinds]]` table per kind of document this repository gives IDs to — `kind = \"FS\"`, then `folder = \"docs/specs\"` or `file = \"requirements.md\"`, and a `title` — and run `grund init` again to list them here.".to_string()
}

/// One Project map link, from `label` to `home`.
fn link(label: &str, home: &str) -> String {
    format!(
        "[{}]({})",
        markdown_link_label(label),
        markdown_link_destination(home)
    )
}

/// The managed block — just the H2 section that `init` appends to, or replaces
/// inside, an existing `AGENTS.md` (§FS-init.2.3.10). The template *is* the block;
/// the H2 line carrying the version is its own begin marker (§FS-init.2.3.1).
/// `workspace` carries the §FS-init.2.3.4.16 discovery and §FS-init.2.3.4.15
/// member section, settled once per `init` invocation by `writers/init_render.rs`
/// and handed to every surface that block is written to.
pub(crate) fn render_agents_append_block(
    name: &str,
    project: &Project,
    run: &Run,
    compiled: &Compiled,
    workspace: &WorkspaceGuidance,
    surface: ConversationSurface,
) -> String {
    let mut rendered = canonical_template_text(AGENTS_TEMPLATE);
    for (placeholder, value) in
        agents_template_substitutions(name, (project, run, compiled), workspace, surface)
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
