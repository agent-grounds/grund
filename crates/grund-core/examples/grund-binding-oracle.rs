//! Core-only parity process; no host converter (§FS-distribution.3.0.3).

#[path = "binding_oracle/canonical.rs"]
mod canonical;
#[path = "binding_oracle/cli.rs"]
mod cli;

use grund_core::{EmbeddingRequest, embedding_call, node_embedding_call, node_request};
use serde_json::{Value, json};
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--metadata") {
        println!(
            "{}",
            json!({"protocolVersion":1,"engineVersion":env!("CARGO_PKG_VERSION"),
            "sourceSha":option_env!("GRUND_BINDINGS_SOURCE_SHA").ok_or("build oracle with GRUND_BINDINGS_SOURCE_SHA")?})
        );
        return Ok(());
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: Value = serde_json::from_str(&input)?;
    if let Some(home) = request["home"].as_str() {
        // SAFETY: the oracle is a fresh process and has not started engine threads.
        // §FS-distribution.3.0.3: isolate only this writer process's user targets.
        unsafe {
            std::env::set_var("HOME", home);
            std::env::set_var("USERPROFILE", home);
            std::env::set_var(
                "XDG_CONFIG_HOME",
                std::path::Path::new(home).join(".config"),
            );
        }
    }
    let operation = request["operation"]
        .as_str()
        .ok_or("missing operation")?
        .to_owned();
    // #469 extends #470's framing; the original root/options Python protocol
    // remains available (§FS-distribution.3.0.3).
    if request.get("root").is_none() {
        let data = node_embedding_call(node_request(
            &operation,
            request["args"].as_array().cloned().unwrap_or_default(),
            &std::env::current_dir()?,
        ));
        io::stdout().lock().write_all(&canonical::encode(&data))?;
        return Ok(());
    }
    let data = embedding_call(EmbeddingRequest {
        operation: operation.clone(),
        root: request["root"].as_str().ok_or("missing root")?.into(),
        explicit: true,
        args: request["args"].as_array().cloned().unwrap_or_default(),
        options: request["options"].clone(),
    });
    let canonical = canonical::encode(&data);
    let mut response = json!({"data":data,"canonical":canonical::base64(&canonical)});
    if operation == "check" {
        let (out, err) = cli::project(&response["data"]);
        response["cli_stdout"] = json!(out);
        response["cli_stderr"] = json!(err);
    }
    writeln!(io::stdout().lock(), "{response}")?;
    Ok(())
}
