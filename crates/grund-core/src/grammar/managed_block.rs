//! The managed-block protocol (§AR-system.2.1): how to find a delimited block
//! grund owns inside a document that is otherwise somebody else's, and read the
//! `vN` version it carries. Three writers splice such a block — the agent
//! entrypoint of §FS-init.2.3, the terminal dotfile of §FS-integrations.4.1 and
//! the user-level agent instructions of §FS-integrations.4.3 — and `grund
//! check`'s agent-entrypoint rule reads the first one back (§FS-check.3.5). The
//! finding is lexical: a marker line, its version, and the byte span between
//! the markers. It names no config, no findings and no client, so it sits at
//! the level every reader of it is above (§AR-system.4).
//!
//! Why all three live here rather than one being derived from the others: they
//! are three spellings of the same protocol, and the differences are real.
//! §FS-init's block is bounded by fixed HTML-comment delimiters matched by the
//! compiled regexes beside this file, carries its version in a `## Grounding
//! with grund (vN)` heading *inside* the block rather than in a delimiter,
//! still recognizes the pre-v4 heading-bounded form, and reports a broken
//! delimiter with the byte offset a diagnostic is line-anchored from
//! ([`AgentsBlockLookup::Malformed`]). §FS-integrations' two blocks put the
//! version in the marker line itself, take the host file's line-comment token
//! as a parameter — `#`, `//` or `--`, because a `#` marker in `wezterm.lua` is
//! a syntax error that costs the user their config — scan whole physical lines
//! rather than matching a regex, have no legacy form, and refuse a duplicate or
//! mismatched pair with a message and no offset. Unifying them is a change of
//! behavior in at least one of the three; collecting them is not
//! (§AR-core-module-layout.1).
//!
//! What stayed with each writer is the half that decides: where a block goes
//! when there is none, what verb the run reports, and what bytes the block
//! carries.

use regex::Match;

use super::compiled::{
    AGENTS_BLOCK_BEGIN, AGENTS_BLOCK_END, AGENTS_BLOCK_H2, AGENTS_SECTION_BOUNDARY,
};

/// v5 (§FS-init.2.3.6.2, §DF-integrations-command, §DF-repo-conversation-opinion):
/// the block gains the `### Clickable citations` section — the fixed
/// repository-web convention, plus a config-derived local-conversation sentence
/// when `[reference] conversation = "link"` is set. v4 (§FS-init.2.3.9,
/// §DF-managed-block-delimiters): explicit `<!-- BEGIN/END GRUND MANAGED BLOCK -->`
/// delimiters replace the implicit H2-to-next-heading region, and the worked
/// citation example is `<§>`-escaped so generated output passes `grund check`
/// unmodified. v3 (§FS-init.2.3.5.9, §DF-citation-directions) replaced the
/// hand-written climbing-rule bullet with a generated `### Citation directions`
/// section derived from `[citations]`.
/// v6 (§FS-init.2.3.6.2, §DF-conversation-link-target): the local-conversation
/// sentence became the gated link form — a Markdown link over the `file` target
/// on the Claude entrypoints, the plain location everywhere else.
/// v7 (§FS-config.1.3, §DF-config-file-location.2.3): the namespace rule tells an
/// agent to give a new subproject a bare `grund.toml` rather than
/// `.agents/grund.toml`. That is the taught workflow changing — an agent
/// following a v6 block creates a config in the form `init` no longer
/// generates — so it carries a version bump rather than a silent rewrite
/// (§FS-init.2.3.7).
/// v8 (§FS-init.2.3.5.9, §DF-directions-render): the generated
/// `### Citation directions` section is re-rendered exactly — a unit per bullet,
/// a grouped conjunction of alternatives, `*/K` said in words, a closed per-kind
/// default folded into its permission, a legend for what gates, and the
/// grounding sentence `[reference] require_grounding` was never rendering. The
/// rules an agent reads changed, so it carries a bump rather than a silent
/// rewrite (§FS-init.2.3.7).
/// v9 (§FS-init.2.3.4.3): the cheap-read ladder gains the point-size sweep so
/// oversized leads are discoverable before an agent pays to read them.
/// v10 (§FS-init.2.3.4.5.1): Markdown declaration bodies teach that ATX headings
/// need section coordinates, with the body/fence/source exemptions and bold
/// label alternative of §FS-check.4.14.1.
/// v11 (§FS-init.2.3.5, §FS-rules.9): projects with a rule kind gain the exact
/// accepted sentences under `### Chapter rules`; projects without one continue
/// to render v10 byte for byte.
pub(crate) const AGENTS_BLOCK_VERSION: u32 = 11;

/// The byte span and `vN` version of the managed block inside an `AGENTS.md`
/// (§FS-init.2.3) — what both `grund init`'s update and `grund check`'s validation
/// (§FS-check.3.5) key off.
pub(crate) struct AgentsBlock {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) version: u32,
}

/// What locating the managed block found (§FS-init.2.3): a well-formed block
/// (delimited or legacy), no block at all, or delimiters that are present but
/// broken — which neither `init` nor `check` may splice over.
pub(crate) enum AgentsBlockLookup {
    Found(AgentsBlock),
    Absent,
    /// `message` names the specific defect; `at` is the byte offset of the
    /// offending delimiter line, for line-anchored diagnostics.
    Malformed {
        message: String,
        at: usize,
    },
}

/// Locate the managed block in an agent entrypoint (§FS-init.2.3). From v4 the
/// block is bounded by explicit `<!-- BEGIN/END GRUND MANAGED BLOCK -->`
/// delimiter lines; a legacy v3-and-earlier block has no delimiters — its H2
/// marker line (`## Grounding with grund (vN)`) opens it and it runs until the
/// next H1 or H2 (or EOF). Broken delimiters are reported as `Malformed`
/// rather than guessed around (§FS-check.3.5.2).
pub(crate) fn find_agents_block(text: &str) -> AgentsBlockLookup {
    let begins: Vec<Match<'_>> = AGENTS_BLOCK_BEGIN.find_iter(text).collect();
    let ends: Vec<Match<'_>> = AGENTS_BLOCK_END.find_iter(text).collect();
    if begins.is_empty() && ends.is_empty() {
        return find_legacy_agents_block(text);
    }
    let Some(begin) = begins.first() else {
        return AgentsBlockLookup::Malformed {
            message: "`<!-- END GRUND MANAGED BLOCK -->` without a begin delimiter".to_string(),
            at: ends[0].start(),
        };
    };
    if begins.len() > 1 {
        return AgentsBlockLookup::Malformed {
            message: "duplicate `<!-- BEGIN GRUND MANAGED BLOCK -->`".to_string(),
            at: begins[1].start(),
        };
    }
    if let Some(stray) = ends.iter().find(|end| end.start() < begin.start()) {
        return AgentsBlockLookup::Malformed {
            message: "`<!-- END GRUND MANAGED BLOCK -->` before the begin delimiter".to_string(),
            at: stray.start(),
        };
    }
    let Some(end) = ends.first() else {
        return AgentsBlockLookup::Malformed {
            message: "missing `<!-- END GRUND MANAGED BLOCK -->`".to_string(),
            at: begin.start(),
        };
    };
    if ends.len() > 1 {
        return AgentsBlockLookup::Malformed {
            message: "duplicate `<!-- END GRUND MANAGED BLOCK -->`".to_string(),
            at: ends[1].start(),
        };
    }
    let region = &text[begin.start()..end.end()];
    let Some(version) = AGENTS_BLOCK_H2
        .captures(region)
        .and_then(|caps| caps.name("version")?.as_str().parse::<u32>().ok())
    else {
        return AgentsBlockLookup::Malformed {
            message: "no `## Grounding with grund (vN)` heading between the delimiters".to_string(),
            at: begin.start(),
        };
    };
    // The span owns the END delimiter's line ending, so splicing a freshly
    // rendered block (which ends `… -->\n`) over an on-disk block reproduces
    // the file byte-for-byte and re-runs stay `exists ` (§FS-init.2.3.1).
    let mut span_end = end.end();
    if text.as_bytes().get(span_end) == Some(&b'\n') {
        span_end += 1;
    }
    AgentsBlockLookup::Found(AgentsBlock {
        start: begin.start(),
        end: span_end,
        version,
    })
}

/// The pre-v4 lookup: the H2 marker line opens the block and the next H1/H2 (or
/// EOF) closes it (§FS-init.2.3.9.2).
fn find_legacy_agents_block(text: &str) -> AgentsBlockLookup {
    let Some(caps) = AGENTS_BLOCK_H2.captures(text) else {
        return AgentsBlockLookup::Absent;
    };
    let (Some(begin_match), Some(version)) = (
        caps.get(0),
        caps.name("version")
            .and_then(|version| version.as_str().parse::<u32>().ok()),
    ) else {
        return AgentsBlockLookup::Absent;
    };
    let after = begin_match.end();
    let section_end = AGENTS_SECTION_BOUNDARY
        .find_at(text, after)
        .map(|m| m.start())
        .unwrap_or(text.len());
    // Trailing blank lines before the next section are inter-section spacing,
    // not part of the managed body. Trim them back so a re-render of the same
    // content is a no-op (`exists `, §FS-init.2.3.1).
    let mut end = section_end;
    while end > after && text[..end].ends_with("\n\n") {
        end -= 1;
    }
    AgentsBlockLookup::Found(AgentsBlock {
        start: begin_match.start(),
        end,
        version,
    })
}

/// The version stamped into the managed dotfile block markers (§FS-integrations.4.1.4).
/// Bumped when an embedded snippet changes in a way a re-run should propagate.
pub const INTEGRATIONS_BLOCK_VERSION: u32 = 1;

/// Version for the user-level agent-instruction block (§FS-integrations.4.3.13).
/// v2 (§DF-repo-conversation-opinion): self-scoping texts — gated on the presence
/// of a `grund.toml`, with the repo-opinion precedence sentence in `plain`.
/// v3 (§DF-conversation-link-target): the `link` text addresses the declaration
/// through `conversation_target`, gated per agent.
/// v4 (§DF-config-file-location): the self-scoping gate names both discovery
/// locations — a repository configured by a bare root `grund.toml` is a grund
/// repository the v3 gate did not describe (§FS-config.1.3).
pub(crate) const AGENT_GUIDANCE_BLOCK_VERSION: u32 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ManagedBlockSpan {
    pub(crate) version: u32,
    pub(crate) start: usize,
    pub(crate) stop: usize,
}

pub(crate) fn integrations_block_markers(comment: &str, version: u32) -> (String, String) {
    (
        format!("{comment} >>> grund integrations (v{version}) >>>"),
        format!("{comment} <<< grund integrations (v{version}) <<<"),
    )
}

/// Find exactly one complete managed block at any supported version. Marker
/// spans are whole physical lines, so accepted indentation cannot leak suffix
/// bytes into the rewritten config (§FS-integrations.4.1.4).
pub(crate) fn find_managed_block(
    comment: &str,
    text: &str,
) -> Result<Option<ManagedBlockSpan>, String> {
    let mut offset = 0;
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']).trim();
        let stop = offset + line.len();
        if let Some(version) = integration_marker_version(comment, trimmed, true) {
            begins.push((version, offset, stop));
        }
        if let Some(version) = integration_marker_version(comment, trimmed, false) {
            ends.push((version, offset, stop));
        }
        offset = stop;
    }
    if begins.is_empty() && ends.is_empty() {
        return Ok(None);
    }
    if begins.len() != 1 || ends.len() != 1 {
        return Err("found incomplete or multiple grund integrations blocks; repair or remove the managed markers, then re-run".to_string());
    }
    let (version, start, _) = begins[0];
    let (end_version, end_start, stop) = ends[0];
    if version > INTEGRATIONS_BLOCK_VERSION {
        return Err(format!(
            "config contains newer grund integrations block v{version}; this binary supports v{INTEGRATIONS_BLOCK_VERSION}"
        ));
    }
    if version != end_version || end_start < start {
        return Err("found mismatched grund integrations block markers; repair or remove the managed block, then re-run".to_string());
    }
    Ok(Some(ManagedBlockSpan {
        version,
        start,
        stop,
    }))
}

fn integration_marker_version(comment: &str, line: &str, begin: bool) -> Option<u32> {
    let (prefix, suffix) = if begin {
        (format!("{comment} >>> grund integrations (v"), ") >>>")
    } else {
        (format!("{comment} <<< grund integrations (v"), ") <<<")
    };
    line.strip_prefix(&prefix)?
        .strip_suffix(suffix)?
        .parse::<u32>()
        .ok()
}

pub(crate) fn agent_guidance_markers(version: u32) -> (String, String) {
    (
        format!("<!-- >>> grund integrations citation rendering (v{version}) >>> -->"),
        format!("<!-- <<< grund integrations citation rendering (v{version}) <<< -->"),
    )
}

pub(crate) fn find_agent_guidance_block(text: &str) -> Result<Option<ManagedBlockSpan>, String> {
    let mut offset = 0;
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']).trim();
        let stop = offset + line.len();
        if let Some(version) = agent_guidance_marker_version(trimmed, true) {
            begins.push((version, offset, stop));
        }
        if let Some(version) = agent_guidance_marker_version(trimmed, false) {
            ends.push((version, offset, stop));
        }
        offset = stop;
    }
    if begins.is_empty() && ends.is_empty() {
        return Ok(None);
    }
    if begins.len() != 1 || ends.len() != 1 {
        return Err("found incomplete or multiple grund citation-rendering blocks; repair or remove the managed markers, then re-run".to_string());
    }
    let (version, start, _) = begins[0];
    let (end_version, end_start, stop) = ends[0];
    if version > AGENT_GUIDANCE_BLOCK_VERSION {
        return Err(format!(
            "instructions contain newer grund citation-rendering block v{version}; this binary supports v{AGENT_GUIDANCE_BLOCK_VERSION}"
        ));
    }
    if version != end_version || end_start < start {
        return Err("found mismatched grund citation-rendering block markers; repair or remove the managed block, then re-run".to_string());
    }
    Ok(Some(ManagedBlockSpan {
        version,
        start,
        stop,
    }))
}

fn agent_guidance_marker_version(line: &str, begin: bool) -> Option<u32> {
    let (prefix, suffix) = if begin {
        (
            "<!-- >>> grund integrations citation rendering (v",
            ") >>> -->",
        )
    } else {
        (
            "<!-- <<< grund integrations citation rendering (v",
            ") <<< -->",
        )
    };
    line.strip_prefix(prefix)?
        .strip_suffix(suffix)?
        .parse::<u32>()
        .ok()
}
