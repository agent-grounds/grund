use super::agents_sections::{stale_sections, workspace_entry_detail};
use crate::config::Frame;
use crate::grammar::{
    AGENTS_BLOCK_END, AGENTS_BLOCK_VERSION, AgentsBlockLookup, AgentsBlockMember,
    WORKSPACE_SECTIONS, agents_block_heading, find_agents_block,
};
use crate::model::remedies;
use crate::model::{
    CheckReport, Code, Diagnostic, Expected, ExpectedEntrypoint, ScopedExpectation,
};
use crate::sources::Sources;
use std::path::Path;

/// One of §FS-errors.3.6.1's final templates around its `detail`: the
/// `repo maintenance: ` classification ahead, and the reminder that the finding
/// leaves citation validity alone behind. A message classification only, not a
/// finding category or selector value (§FS-check.1).
fn agents_init_message(detail: String) -> String {
    format!("repo maintenance: {detail} (does not affect citation validity)")
}

/// §FS-errors.3.6.1: a held-stable template that names `grund init`, grown at
/// its end with the directory of another project's block.
fn command_message(detail: String, owner: Option<&str>) -> String {
    let message = agents_init_message(detail);
    match owner {
        Some(dir) => format!("{message}; {}", remedies::agents_init_elsewhere(dir)),
        None => message,
    }
}

/// The text of both stale templates ahead of the section noun, and the text
/// after it, which [`stale_block`] writes and [`stale_section_noun`] reads back
/// (§FS-errors.3.6.1).
const STALE_BEFORE_NOUN: &str = "stale grund init block: the ";
const STALE_AFTER_NOUN: &str = " section differs from ";

/// The section noun a stale finding's `message` names (§FS-check.3.5.3), or
/// `None` for any other `agents-init` variant.
pub(crate) fn stale_section_noun(message: &str) -> Option<&str> {
    let rest = message
        .strip_prefix("repo maintenance: ")?
        .strip_prefix(STALE_BEFORE_NOUN)?;
    rest.split_once(STALE_AFTER_NOUN).map(|(noun, _)| noun)
}

/// Validate the managed agent-entrypoint blocks (§FS-check.3.5): the begin/end
/// marker pair must be present and intact, and the `vN` version must match this
/// binary — an older `vN` is "run `grund init`" (§FS-init.2.3.7), a newer one is
/// fatal. `AGENTS.md` is canonical; known companion entrypoints are checked when
/// present and not symlinked to `AGENTS.md`.
///
/// What each block should say arrives as `Expected` (§AR-checker.1.3): its
/// version, the entrypoints in comparison order, each root one's whole block
/// cut into sections and each scoped one's sections, rendered for its own
/// surface, whether the workspace sections are compared (§FS-check.3.5.5), and
/// the companion probe's io error.
/// This rule compares those bytes with disk and renders nothing
/// (§AR-checker.2.7). §FS-check.3.5.4's chapter-rule section is among them
/// exactly when the writers could render it, which is every case but a
/// genuinely invalid rule — and that one carries its own located error at the
/// rule's heading, from `check_chapter_rules`.
pub(super) fn check_agents_block_version(
    expected: &Expected,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    // §FS-check.3.5.3: another project's block names the directory it belongs to.
    let owner = (frame.run.root != frame.display.root).then(|| frame.display_path(&frame.run.root));
    let held = Held {
        version: expected.block_version,
        workspace_sections: expected.workspace_sections,
        owner: owner.as_deref(),
    };
    for entrypoint in &expected.entrypoints {
        check_agent_block_path(entrypoint, held, frame.sources(), report);
    }
    if let Some((path, message)) = &expected.entrypoint_probe_error {
        report.errors.push(Diagnostic {
            code: Code::IO,
            path: Some(path.clone()),
            line: Some(1),
            column: None,
            message: message.clone(),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
}

/// What every block of one project is held to: the version, whether the two
/// workspace sections are compared (§FS-check.3.5.5), and the directory a
/// finding about another project's block names (§FS-check.3.5.3).
#[derive(Clone, Copy)]
pub(crate) struct Held<'a> {
    pub(crate) version: u32,
    pub(crate) workspace_sections: bool,
    pub(crate) owner: Option<&'a str>,
}

/// Checks one agent-entrypoint file's managed `grund init` block: present when
/// required, version supported, and every section still matching the bytes
/// `Expected` carries for this file.
///
/// Why the generated sections are compared rather than re-derived: rendering is
/// deterministic, so a fresh render is the hash — and the writers render it
/// through the renderers `init` writes with (§AR-system.2.11, §AR-checker.2.7).
/// Workspace findings retain the compared bytes for §FS-check.3.5.3.1's detail.
/// `\r` is stripped from the block first, because the managed `AGENTS.md` is not
/// pinned to LF in `.gitattributes`, so a Windows checkout has CRLF and would
/// read as drift against the LF render.
pub(crate) fn check_agent_block_path(
    entrypoint: &ExpectedEntrypoint,
    held: Held<'_>,
    sources: Sources<'_>,
    report: &mut CheckReport,
) {
    let (expected_version, owner) = (held.version, held.owner);
    let path = entrypoint.path.as_path();
    let require_block = entrypoint.require_block;
    // §FS-check.6.1.3: retain this probe when the entrypoint is absent.
    crate::config::observe_input(path, false);
    // §FS-check.6.1.1: observed before its read, as a save would write it
    // (§AR-system.2.13).
    if !sources.exists(path) {
        // §FS-init.2.3.5.8: a destination of the live render with no file is a
        // missing block.
        if matches!(entrypoint.scoped, Some(ScopedExpectation::Block(_))) {
            report
                .errors
                .push(missing_block(path, expected_version, owner));
        }
        return;
    }
    let Ok(text) = sources.text(path) else {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent entrypoint");
        report.errors.push(Diagnostic {
            code: Code::IO,
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
                code: Code::AGENTS_INIT,
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
    if let Some(block) = block {
        // §FS-check.4.9.1: the absent-member warning says what such a run left out.
        report.verified_blocks = true;
        let line = line_for_byte_index(&text, block.start);
        let block_text = text[block.start..block.end].replace('\r', "");
        if let Some(ScopedExpectation::Orphan) = entrypoint.scoped {
            // §FS-init.2.3.5.13.1: an orphan still carrying a block is stale.
            report
                .errors
                .push(stale_block(path, line, "scoped guidance", owner, None));
            return;
        }
        if block.version < expected_version {
            report.errors.push(Diagnostic {
                code: Code::AGENTS_INIT,
                path: Some(path.to_path_buf()),
                line: Some(line),
                column: None,
                message: command_message(
                    format!(
                        "outdated grund init block v{} — {}",
                        block.version,
                        remedies::agents_init_outdated(expected_version)
                    ),
                    owner,
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        } else if block.version > AGENTS_BLOCK_VERSION {
            // §FS-init.2.3.7.3.1: only above the maximum, which the message names.
            report.errors.push(Diagnostic {
                code: Code::AGENTS_INIT,
                path: Some(path.to_path_buf()),
                line: Some(line),
                column: None,
                message: agents_init_message(format!(
                    "unsupported grund init block v{} — this grund supports v{}",
                    block.version, AGENTS_BLOCK_VERSION
                )),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        } else {
            // §FS-check.3.5.4: the version marker alone misses a config edit, so
            // compare every section against `Expected`; the scoped file's own
            // sections are held as before, then its whole block.
            let mut stale = false;
            if entrypoint.scoped.is_none() {
                let current = block_text.replacen(
                    &agents_block_heading(block.version),
                    &agents_block_heading(expected_version),
                    1,
                );
                for section in
                    stale_sections(&entrypoint.sections, &current, held.workspace_sections)
                {
                    stale = true;
                    report.errors.push(stale_block(
                        path,
                        line,
                        &section.noun,
                        owner,
                        Some((section.expected, section.current)),
                    ));
                }
            } else {
                for section in &entrypoint.sections {
                    let current = section_in_block(&block_text, &section.heading).unwrap_or("");
                    if current != section.bytes.trim_end() {
                        stale = true;
                        report.errors.push(stale_block(
                            path,
                            line,
                            &section.noun,
                            owner,
                            Some((&section.bytes, current)),
                        ));
                    }
                }
            }
            // §FS-init.2.3.7.3.2: another known member is stale even when no
            // compared section tells it apart, named by what its selection turns on.
            if !stale && block.version != expected_version {
                stale = true;
                let noun = unselected_member_noun(block.version, expected_version);
                report
                    .errors
                    .push(stale_block(path, line, noun, owner, None));
            }
            // §FS-init.2.3.5.12: a scoped block is the render, byte for byte, so
            // a difference no section names is still a stale one.
            if let Some(ScopedExpectation::Block(bytes)) = &entrypoint.scoped
                && !stale
                && block_text.trim_end() != bytes.trim_end()
            {
                report
                    .errors
                    .push(stale_block(path, line, "scoped guidance", owner, None));
            }
        }
        return;
    }
    if !require_block {
        return;
    }
    report
        .errors
        .push(missing_block(path, expected_version, owner));
}

/// The section a known block revision other than the selected one turns on
/// (§FS-init.2.3.7.3.2): `scoped guidance` across the pairs, alone where the
/// rule-kind member differs too, otherwise `chapter rules`.
fn unselected_member_noun(version: u32, expected_version: u32) -> &'static str {
    let (found, selected) = (
        AgentsBlockMember::of(version),
        AgentsBlockMember::of(expected_version),
    );
    if found.scoped != selected.scoped {
        "scoped guidance"
    } else {
        "chapter rules"
    }
}

/// §FS-check.3.5.3: a block that should be there and is not.
fn missing_block(path: &Path, expected_version: u32, owner: Option<&str>) -> Diagnostic {
    Diagnostic {
        code: Code::AGENTS_INIT,
        path: Some(path.to_path_buf()),
        line: Some(1),
        column: None,
        message: command_message(
            format!(
                "missing grund init block v{expected_version} — {}",
                remedies::agents_init_missing()
            ),
            owner,
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §FS-check.3.5.3: a block whose `noun` differs from the render. A workspace
/// section differs from the workspace and names its directory inside the
/// template; every other section differs from `grund.toml`, and the held-stable
/// template grows at its end (§FS-errors.3.6.1).
/// Workspace sections explain their first unequal body entry (§FS-check.3.5.3.1).
pub(super) fn stale_block(
    path: &Path,
    line: usize,
    noun: &str,
    owner: Option<&str>,
    bytes: Option<(&str, &str)>,
) -> Diagnostic {
    let message = if WORKSPACE_SECTIONS.contains(&noun) {
        let refresh = owner.map_or_else(remedies::agents_init_refresh, |dir| {
            remedies::agents_init_refresh_in(dir)
        });
        let (expected, current) = bytes.unwrap_or(("", ""));
        let detail = workspace_entry_detail(expected, current);
        agents_init_message(format!(
            "{STALE_BEFORE_NOUN}{noun}{STALE_AFTER_NOUN}the workspace ({detail}) — {refresh}"
        ))
    } else {
        command_message(
            format!(
                "{STALE_BEFORE_NOUN}{noun}{STALE_AFTER_NOUN}grund.toml — {}",
                remedies::agents_init_refresh()
            ),
            owner,
        )
    };
    Diagnostic {
        code: Code::AGENTS_INIT,
        path: Some(path.to_path_buf()),
        line: Some(line),
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The text of a `heading`-led section inside the managed block, from the heading
/// line to the next heading of any level (or block end), trailing blank lines
/// trimmed. Used to byte-compare a scoped file's sections against `Expected`
/// (§FS-check.3.5) and to read the root's `### Scoped guidance`. The boundary is *any* following heading, not just H1/H2, so
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
