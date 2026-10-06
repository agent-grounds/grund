//! Writer defaults and complete outcomes (§FS-distribution.3.3.6).

use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::{error_data, failure};
use crate::*;
use serde_json::{Value, json};

pub(super) fn write(r: &EmbeddingRequest) -> Result<Value, Value> {
    match r.operation.as_str() {
        "fmt" => {
            let (cautions, result) = format_references_with_run_warnings(FmtOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
                write: r.flag("write"),
                add_marker: r.flag("marker"),
                cross_refs: r.flag("cross_refs"),
            });
            let out = result.map_err(|e| error_data(e, &cautions))?;
            Ok(
                json!({"changes":out.changes.data(),"scan_errors":out.scan_errors.data(),
                "refused_writes":out.refused_writes,"run_cautions":out.warnings.data()}),
            )
        }
        "init" => {
            let mut agents = InitAgentEntrypointSelection::default();
            for agent in r.strings("agents") {
                match agent.as_str() {
                    "canonical" | "codex" | "agents" => agents.canonical = true,
                    "claude" => agents.claude = true,
                    "gemini" => agents.gemini = true,
                    "pi" => agents.pi = true,
                    "copilot" => agents.copilot = true,
                    "cursor" => agents.cursor = true,
                    "windsurf" => agents.windsurf = true,
                    "zed" => agents.zed = true,
                    _ => {}
                }
            }
            let (result, diagnostic, cautions) = crate::writers::init_with_diagnostics(InitOpts {
                target: r.root.clone(),
                name: r.string("name"),
                description: r.string("description"),
                docs: r.flag("docs"),
                force: r.flag("force"),
                dry_run: !r.flag("write"),
                check: r.flag("check"),
                no_vcs: r.flag("no_vcs"),
                agent_selection: agents,
            });
            result.map(|out| out.data()).map_err(|e| {
                let mut f = match diagnostic {
                    Some(source) => error_data(source, &cautions),
                    None => failure("operation", "init", e.message, &cautions),
                };
                f["partial_output"] = e.output.data();
                f
            })
        }
        "fetch" => {
            // §FS-distribution.3.3.6: host validation refuses before this entry;
            // also protect language-neutral callers from accidental execution.
            if !r.flag("write") {
                return Err(failure(
                    "operation",
                    "write-required",
                    "fetch requires write=True".into(),
                    &[],
                ));
            }
            let (cautions, out) = crate::writers::fetch_with_diagnostics(r.arg(0), &r.root);
            out.map_err(|e| error_data(e, &cautions))?;
            Ok(json!({"run_cautions":cautions.data()}))
        }
        "integrations" => crate::writers::integrations_data(
            r.arg(0),
            r.flag("write"),
            r.string("conversation").as_deref(),
            r.string("conversation_target").as_deref(),
            r.string("agent").as_deref(),
        )
        .map_err(|e| error_data(e, &[])),
        _ => unreachable!("writer selected by core dispatch"),
    }
}
