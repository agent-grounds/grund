use crate::config::Frame;
use crate::grammar::{
    AGENTS_BLOCK_BASE_VERSION, AGENTS_BLOCK_END, AGENTS_BLOCK_VERSION, AgentsBlockLookup,
    find_agents_block,
};
use crate::model::{
    CheckReport, Code, Diagnostic, Expected, ExpectedEntrypoint, ScopedExpectation,
};
use crate::sources::Sources;
use std::path::Path;

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
/// What each block should say arrives as `Expected` (§AR-checker.1.3): its
/// version, the entrypoints in comparison order, each one's config-derived
/// sections rendered for its own surface, and the companion probe's io error.
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
    for entrypoint in &expected.entrypoints {
        check_agent_block_path(entrypoint, expected.block_version, frame.sources(), report);
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

/// Checks one agent-entrypoint file's managed `grund init` block: present when
/// required, version supported, and its generated sections still matching the
/// bytes `Expected` carries for this file.
///
/// Why the generated sections are compared rather than re-derived: rendering is
/// deterministic, so a fresh render is the hash — and the writers render it
/// through the renderers `init` writes with (§AR-system.2.11, §AR-checker.2.7).
/// `\r` is stripped from the block first, because the managed `AGENTS.md` is not
/// pinned to LF in `.gitattributes`, so a Windows checkout has CRLF and would
/// read as drift against the LF render.
pub(crate) fn check_agent_block_path(
    entrypoint: &ExpectedEntrypoint,
    expected_version: u32,
    sources: Sources<'_>,
    report: &mut CheckReport,
) {
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
            report.errors.push(missing_block(path, expected_version));
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
        let line = line_for_byte_index(&text, block.start);
        let block_text = text[block.start..block.end].replace('\r', "");
        if let Some(ScopedExpectation::Orphan) = entrypoint.scoped {
            // §FS-init.2.3.5.13.1: an orphan still carrying a block is stale.
            report
                .errors
                .push(stale_block(path, line, "scoped guidance"));
            return;
        }
        if block.version < expected_version {
            report.errors.push(Diagnostic {
                code: Code::AGENTS_INIT,
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
        } else if block.version > AGENTS_BLOCK_VERSION
            || newer_than_selected(block.version, expected_version)
        {
            report.errors.push(Diagnostic {
                code: Code::AGENTS_INIT,
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
            // §FS-check.3.5 / §FS-init.2.3.5.8: the version marker alone misses a
            // config edit, so byte-compare each section against `Expected`; one
            // expected empty must be absent.
            let mut stale = false;
            for section in &entrypoint.sections {
                if section_in_block(&block_text, section.heading).unwrap_or("")
                    != section.bytes.trim_end()
                {
                    stale = true;
                    report.errors.push(stale_block(path, line, section.noun));
                }
            }
            // §FS-init.2.3.7.1: a scoped-pair block where the unscoped pair is
            // selected is stale even when no compared section tells it apart.
            if !stale && block.version != expected_version {
                stale = true;
                report
                    .errors
                    .push(stale_block(path, line, "scoped guidance"));
            }
            // §FS-init.2.3.5.12: a scoped block is the render, byte for byte, so
            // a difference no section names is still a stale one.
            if let Some(ScopedExpectation::Block(bytes)) = &entrypoint.scoped
                && !stale
                && block_text.trim_end() != bytes.trim_end()
            {
                report
                    .errors
                    .push(stale_block(path, line, "scoped guidance"));
            }
        }
        return;
    }
    if !require_block {
        return;
    }
    report.errors.push(missing_block(path, expected_version));
}

/// Whether a supported block is newer than the version the config selects. A
/// v41 or v42 block where v39 or v40 is selected is not: it is stale, not
/// unsupported (§FS-init.2.3.7.1).
fn newer_than_selected(version: u32, expected_version: u32) -> bool {
    let scoped = AGENTS_BLOCK_BASE_VERSION + 2;
    let scoped_where_unscoped = version >= scoped && expected_version < scoped;
    !scoped_where_unscoped && version > expected_version
}

/// §FS-check.3.5.3: a block that should be there and is not.
fn missing_block(path: &Path, expected_version: u32) -> Diagnostic {
    Diagnostic {
        code: Code::AGENTS_INIT,
        path: Some(path.to_path_buf()),
        line: Some(1),
        column: None,
        message: agents_init_message(format!(
            "missing grund init block v{expected_version} — run `grund init` to install it"
        )),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §FS-check.3.5.3: a block whose `noun` differs from the render.
fn stale_block(path: &Path, line: usize, noun: &str) -> Diagnostic {
    Diagnostic {
        code: Code::AGENTS_INIT,
        path: Some(path.to_path_buf()),
        line: Some(line),
        column: None,
        message: agents_init_message(format!(
            "stale grund init block: {noun} differ from grund.toml — run `grund init` to refresh"
        )),
        sites: Vec::new(),
        authority: Vec::new(),
    }
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
