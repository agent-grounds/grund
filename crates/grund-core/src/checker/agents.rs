use crate::grammar::{
    AGENTS_BLOCK_END, AGENTS_BLOCK_VERSION, AgentsBlockLookup, find_agents_block,
};
use crate::model::{CheckReport, Diagnostic, Expected, ExpectedEntrypoint};

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
pub(super) fn check_agents_block_version(expected: &Expected, report: &mut CheckReport) {
    for entrypoint in &expected.entrypoints {
        check_agent_block_path(entrypoint, expected.block_version, report);
    }
    if let Some((path, message)) = &expected.entrypoint_probe_error {
        report.errors.push(Diagnostic {
            code: "io",
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
    report: &mut CheckReport,
) {
    let path = entrypoint.path.as_path();
    let require_block = entrypoint.require_block;
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
            // §FS-check.3.5 / §FS-init.2.3.5.8: the generated sections follow
            // config, so the version marker alone cannot catch a config edit that
            // left the block stale. Byte-compare each against `Expected`.
            let block_text = text[block.start..block.end].replace('\r', "");
            for section in &entrypoint.sections {
                if section_in_block(&block_text, section.heading) != Some(section.bytes.trim_end())
                {
                    let noun = section.noun;
                    report.errors.push(Diagnostic {
                        code: "agents-init",
                        path: Some(path.to_path_buf()),
                        line: Some(line),
                        column: None,
                        message: agents_init_message(format!(
                            "stale grund init block: {noun} differ from grund.toml — run `grund init` to refresh"
                        )),
                        sites: Vec::new(),
                        authority: Vec::new(),
                    });
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
