use std::path::Path;

use crate::config::Config;
use crate::grammar::{
    AGENTS_BLOCK_END, AGENTS_BLOCK_VERSION, AgentsBlockLookup, find_agents_block, parse_id_arg,
};
use crate::model::{Catalog, CheckReport, Diagnostic};
use crate::resolver::{WorkspaceCheckTarget, markdown_link_target};
use crate::scanner::companion_agent_entrypoints;
use crate::templates::{
    ConversationSurface, citation_directions_section, clickable_citations_section,
};
use std::collections::BTreeMap;

/// One of §FS-errors.3.6.1's five final templates around its `detail`: the
/// `repo maintenance: ` classification ahead, and the reminder that the finding
/// leaves citation validity alone behind. A message classification only, not a
/// finding category or selector value (§FS-check.1).
fn agents_init_message(detail: String) -> String {
    format!("repo maintenance: {detail} (does not affect citation validity)")
}

/// Validate the managed agent-entrypoint blocks (§FS-check.3.5): the begin/end
/// marker pair must be present and intact, and the `vN` version must match this
/// binary — an older `vN` is "run `grund init`" (§FS-init.2.3.7), a newer one is
/// fatal. `AGENTS.md` is canonical; known companion entrypoints are checked when
/// present and not symlinked to `AGENTS.md`.
///
/// §FS-check.3.5.4: the `### Chapter rules` section is re-rendered and compared
/// whether or not every configured rule resolved. A rule this scope cannot
/// verify still earns its bullet (§FS-rules.4.1.2), so the comparison is dropped
/// only for a genuinely invalid rule — the one case that already carries its own
/// located error at the rule's heading. The diagnostics are not re-emitted here:
/// `check_chapter_rules` reports them at that site.
///
/// `findings` is the run's resolution scope (§FS-check.1.3.6.1), never its report
/// scope, so a path holding no rule declaration still renders every chapter-rule
/// bullet and a narrowed run compares against the render `grund check .` compares
/// against (§FS-rules.9.1.1).
///
/// `workspace` is the project map the run loaded, which is what lets a run at the
/// workspace root resolve a member's cross-boundary rule and so catch a member
/// block missing its bullet (§FS-rules.9.1).
pub(super) fn check_agents_block_version(
    findings: &Catalog,
    config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    let rule_rows = config
        .kinds
        .iter()
        .any(|kind| kind.rules)
        .then(|| {
            let vocab = super::workspace_vocabulary(config, workspace);
            super::configured_rule_sentences(findings, config, &vocab).ok()
        })
        .flatten()
        .map(|rules| rules.rows);
    let root = &config.root;
    // §FS-init.2.3.5.10: render destinations from the same loaded declarations
    // that supplied the validated, ordered rule sentences.
    let rule_guidance = rule_rows.as_deref().map(|rows| (findings, rows));
    let canonical = root.join("AGENTS.md");
    let canonical_exists = canonical.exists();
    if canonical_exists {
        check_agent_block_path_with_rules(config, &canonical, report, true, rule_guidance);
    }
    match companion_agent_entrypoints(root) {
        Ok(companions) => {
            for companion in companions {
                check_agent_block_path_with_rules(
                    config,
                    &companion,
                    report,
                    canonical_exists,
                    rule_guidance,
                );
            }
        }
        Err((path, message)) => {
            report.errors.push(Diagnostic {
                code: "io",
                path: Some(path),
                line: Some(1),
                column: None,
                message,
                sites: Vec::new(),
                authority: Vec::new(),
            });
        }
    }
}

/// Checks one agent-entrypoint file's managed `grund init` block: present when
/// required, version supported, and its generated sections still matching config.
///
/// Why the generated sections are compared by re-rendering: rendering is
/// deterministic, so a fresh render is the hash — and what a managed block
/// should say is a function of config alone, which is why `templates/` holds
/// both renderers and `init` writes exactly what this compares against
/// (§AR-system.2.11, §AR-checker.2.7). `\r` is stripped from the block
/// first, because the managed `AGENTS.md` is not pinned to LF in `.gitattributes`,
/// so a Windows checkout has CRLF and would read as drift against the LF render.
///
/// Why the local-conversation sentence is re-rendered per file: flipping
/// `[reference] conversation` without re-running `grund init` must surface as
/// drift, and the sentence also varies by entrypoint — so the comparison derives
/// the surface from the path, the same way `init` chose it.
#[cfg(test)]
pub(crate) fn check_agent_block_path(
    config: &Config,
    path: &Path,
    report: &mut CheckReport,
    require_block: bool,
) {
    check_agent_block_path_with_rules(config, path, report, require_block, None);
}

fn check_agent_block_path_with_rules(
    config: &Config,
    path: &Path,
    report: &mut CheckReport,
    require_block: bool,
    rule_guidance: Option<(&Catalog, &[(String, String)])>,
) {
    // §FS-check.6.1.3: retain this probe when the entrypoint is absent.
    crate::config::observe_input(path, false);
    if !path.exists() {
        return;
    }
    // §FS-check.6.1.1: cover this effective input before its shared read.
    let Ok(text) = crate::config::input_read_to_string(path) else {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent entrypoint");
        report.errors.push(Diagnostic {
            code: "io",
            path: Some(path.to_path_buf()),
            line: Some(1),
            column: None,
            message: format!("cannot read {file_name}"),
            sites: Vec::new(),
            authority: Vec::new(),
        });
        return;
    };
    let block = match find_agents_block(&text) {
        AgentsBlockLookup::Malformed { message, at } => {
            // §FS-check.3.5.2 / §FS-init.2.3.11: broken delimiters are diagnosed at
            // the offending line and never rewritten — `grund init` refuses
            // them too.
            report.errors.push(Diagnostic {
                code: "agents-init",
                path: Some(path.to_path_buf()),
                line: Some(line_for_byte_index(&text, at)),
                column: None,
                message: agents_init_message(format!("malformed grund managed block: {message}")),
                sites: Vec::new(),
                authority: Vec::new(),
            });
            return;
        }
        AgentsBlockLookup::Found(block) => Some(block),
        AgentsBlockLookup::Absent => None,
    };
    let expected_version = if config.kinds.iter().any(|kind| kind.rules) {
        15
    } else {
        14
    };
    if let Some(block) = block {
        let line = line_for_byte_index(&text, block.start);
        if block.version < expected_version {
            report.errors.push(Diagnostic {
                code: "agents-init",
                path: Some(path.to_path_buf()),
                line: Some(line),
                column: None,
                message: agents_init_message(format!(
                    "outdated grund init block v{} — run `grund init` to update to v{}",
                    block.version, expected_version
                )),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        } else if block.version > AGENTS_BLOCK_VERSION || block.version > expected_version {
            report.errors.push(Diagnostic {
                code: "agents-init",
                path: Some(path.to_path_buf()),
                line: Some(line),
                column: None,
                message: agents_init_message(format!(
                    "unsupported grund init block v{} — this grund supports v{}",
                    block.version, expected_version
                )),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        } else {
            // §FS-check.3.5 / §FS-init.2.3.5.8: citation directions are generated
            // from `[citations]`, so the version marker alone cannot catch a
            // config edit that left the block stale. Re-render and byte-compare.
            let block_text = text[block.start..block.end].replace('\r', "");
            let mut generated_sections = vec![
                (
                    "### Citation directions",
                    citation_directions_section(config.project()),
                    "citation directions",
                ),
                // §FS-init.2.3.6.1: the local-conversation sentence derives from
                // `[reference] conversation` and varies by entrypoint
                // (§FS-init.2.3.4.17.2), so drift re-renders for *this* file's surface.
                (
                    "### Clickable citations",
                    clickable_citations_section(
                        config.project(),
                        ConversationSurface::for_entrypoint(path),
                    ),
                    "clickable citations",
                ),
            ];
            if let Some((findings, rows)) = rule_guidance {
                generated_sections.push((
                    "### Chapter rules",
                    chapter_rules_section(config, path, findings, rows),
                    "chapter rules",
                ));
            }
            for (heading, expected, noun) in generated_sections {
                if section_in_block(&block_text, heading) != Some(expected.trim_end()) {
                    report.errors.push(Diagnostic {
                        code: "agents-init",
                        path: Some(path.to_path_buf()),
                        line: Some(line),
                        column: None,
                        message: agents_init_message(format!(
                            "stale grund init block: {noun} differ from grund.toml — run `grund init` to refresh"
                        )),
                        sites: Vec::new(),
                    authority: Vec::new(),});
                }
            }
        }
        return;
    }
    if !require_block {
        return;
    }
    report.errors.push(Diagnostic {
        code: "agents-init",
        path: Some(path.to_path_buf()),
        line: Some(1),
        column: None,
        message: agents_init_message(format!(
            "missing grund init block v{} — run `grund init` to install it",
            expected_version
        )),
        sites: Vec::new(),
        authority: Vec::new(),
    });
}

/// The shared chapter-rule section for `init` and its exact drift comparison
/// (§FS-init.2.3.5.10). Preserve ordered authored titles and use the formatter's
/// destination/anchor resolver for live citations relative to this entrypoint.
/// Only the citation is rendered; no formatter runs over the authored sentence.
pub(crate) fn chapter_rules_section(
    config: &Config,
    path: &Path,
    findings: &Catalog,
    rows: &[(String, String)],
) -> String {
    let mut section = String::from(
        "### Chapter rules\n\n`must`/`must not` are `grund check` errors; `should`/`should not` are suggestions (`grund check --suggestions`).\n\n",
    );
    for (origin, sentence) in rows {
        let citation = format!("{}{origin}", config.marker);
        let target = config
            .fmt_cross_refs_enabled
            .then(|| {
                let (id, _) = parse_id_arg(origin, &config.grammar).ok()?;
                markdown_link_target(path, &id, None, config, findings)
            })
            .flatten();
        let citation = match target {
            Some(target) => format!("[{citation}]({target})"),
            None => citation,
        };
        section.push_str(&format!("- {sentence} {citation}\n"));
    }
    section
}

/// The text of a `heading`-led section inside the managed block, from the heading
/// line to the next heading of any level (or block end), trailing blank lines
/// trimmed. Used to byte-compare config-derived sections against a fresh render
/// (§FS-check.3.5). The boundary is *any* following heading, not just H1/H2, so
/// two adjacent config-derived `###` sections (Citation directions, Clickable
/// citations — §FS-init.2.3.5/2.3.6) do not bleed into each other; neither
/// rendered section contains a `#`-led line, so this cannot cut one short.
pub(crate) fn section_in_block<'a>(block_text: &'a str, heading: &str) -> Option<&'a str> {
    let start = block_text.match_indices(heading).find_map(|(index, _)| {
        let at_line_start = index == 0 || block_text.as_bytes().get(index - 1) == Some(&b'\n');
        let after = index + heading.len();
        let line_ends =
            after == block_text.len() || block_text.as_bytes().get(after) == Some(&b'\n');
        (at_line_start && line_ends).then_some(index)
    })?;
    let section_body_start = start + heading.len();
    // A block-final section is bounded by the `<!-- END GRUND MANAGED BLOCK -->`
    // line, not the block end: the delimiter is the block's frame, never part
    // of a rendered section's body.
    let end = [
        next_heading_offset(block_text, section_body_start),
        AGENTS_BLOCK_END
            .find_at(block_text, section_body_start)
            .map(|m| m.start()),
    ]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(block_text.len());
    Some(block_text[start..end].trim_end())
}

/// Offset of the next heading line (any `#`-led line) at or after `from`, scanning
/// line by line so a `#` mid-line never counts.
fn next_heading_offset(text: &str, from: usize) -> Option<usize> {
    let mut offset = from;
    for line in text[from..].split_inclusive('\n') {
        if line.trim_start().starts_with('#') {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn line_for_byte_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
