//! The body of an end-to-end case declaration (§AR-system.2.10): the case
//! manifest §FS-show.2.4 answers an ID query with — the invocation, the expected
//! exit and the fixture list, plus the JSON shape — for the one kind whose
//! declaration is a directory rather than a heading (§AR-scanner.6).
//!
//! It sits with the body slicer rather than with `show` because it is the same
//! answer for a different source: `point_body.rs` asks it for a case's lead and
//! full body exactly as it asks the slicer for a text declaration's, and a
//! measurement that could not reach it would measure nothing for every E2E row
//! (§FS-list.3.4). The bytes it renders are the query's, which is why every
//! sentence of it is unchanged from `queries/show.rs`, where it sat while the
//! queries were its only reader.

use anyhow::{Result, anyhow};

use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::model::{E2eCase, Id, ShowOutput, ShowRenderMode, format_path, json_escape};

/// Render an e2e case as an ID-query body: the invocation, expected exit, and
/// fixture list (or just the invocation with `--brief`), plus the JSON shape — the
/// case manifest of §FS-show.2.4. E2E declarations have no sections, so any
/// `.<section>` is "section not found".
pub(crate) fn show_e2e_case(
    config: &Config,
    path_config: &Config,
    id: &Id,
    case: &E2eCase,
    section: Option<&str>,
    mode: ShowRenderMode,
) -> Result<ShowOutput> {
    if let Some(section) = section {
        return Err(anyhow!(
            "section not found: {}{}{}",
            render_id(config, id),
            config.section_separator,
            section
        ));
    }
    let invocation = format!("grund {}", case.args.join(" "));
    let brief_body = format!("{invocation}\n");
    let manifest = {
        let mut lines = vec![
            invocation.clone(),
            format!("expected exit: {}", case.expected_exit),
            "fixtures:".to_string(),
        ];
        lines.extend(
            case.fixtures
                .iter()
                .map(|path| format!("- {}", format_path(path))),
        );
        format!("{}\n", lines.join("\n"))
    };
    let body = match mode {
        ShowRenderMode::Brief => brief_body,
        ShowRenderMode::Outline => String::new(),
        ShowRenderMode::Default | ShowRenderMode::Toc | ShowRenderMode::Full => manifest,
    };
    let args_json = case
        .args
        .iter()
        .map(|arg| format!("\"{}\"", json_escape(arg)))
        .collect::<Vec<_>>()
        .join(",");
    let fixtures_json = case
        .fixtures
        .iter()
        .map(|path| format!("\"{}\"", json_escape(&format_path(path))))
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        "{{\"id\":\"{}\",\"kind\":\"E2E\",\"path\":\"{}\",\"args\":[{}],\"expected_exit\":{},\"fixtures\":[{}]}}",
        json_escape(&render_id(config, id)),
        // path_config, not config: an `<alias>/E2E-x` shown from a workspace
        // root must report the same root-relative path as every other kind
        // (§FS-workspace.8.1) — this baked JSON bypasses render_show_output_json.
        json_escape(&display_path(path_config, &case.dir)),
        args_json,
        case.expected_exit,
        fixtures_json
    );
    Ok(ShowOutput {
        body,
        path: case.dir.clone(),
        line: 1,
        json: Some(json),
        sections: Vec::new(),
    })
}
