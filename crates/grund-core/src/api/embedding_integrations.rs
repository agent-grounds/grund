//! Node integration records over core install policy (§FS-distribution.3.2.2.2).
use crate::*;
use serde_json::{Value, json};

pub(super) fn project(out: &mut Value, client: Option<&str>, write: bool) {
    let descriptor = |v: &Value| {
        let client = IntegrationClient::from_name(v["client"].as_str().unwrap_or("")).unwrap();
        json!({"client":v["client"],"kind":if client.is_terminal(){"terminal"}else{"editor"},
            "detected":v["detected"],
            "installed":if client.install_kind()==InstallKind::Manual {Value::Null}else{v["installed"].clone()},
            "install_kind":v["install_kind"],"config_target":client.config_target(),
            "resolver_target":if client.is_terminal(){Some(RESOLVER_TARGET)}else{None}})
    };
    let cautions = out["run_cautions"].clone();
    if write {
        let config = user_grund_config_path()
            .and_then(|p| read_optional_text(&p).ok())
            .map(|t| scan_user_config(&t));
        let preferences = config.map_or_else(
            || {
                json!({"conversation":"plain",
            "conversation_target":"path","agent_overrides":{}})
            },
            |c| {
                json!({
                "conversation":c.preference.unwrap_or(ConversationRendering::Plain).name(),
                "conversation_target":c.target.unwrap_or_default().name(),
                "agent_overrides":c.agent_targets.iter().map(|(a,t)|(a.clone(),json!(t.name())))
                    .collect::<serde_json::Map<_,_>>()})
            },
        );
        let notes = out["events"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| e["note"].as_str())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        *out = json!({"mode":"install","events":out["events"].as_array().into_iter().flatten()
            .map(|e|json!({"verb":e["verb"],"path":e["path"]})).collect::<Vec<_>>(),
            "notes":notes,"preferences":preferences,"manual_steps":out["manual_steps"],
            "run_cautions":cautions});
    } else if let Some(name) = client {
        let record = out["clients"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["client"] == name)
            .unwrap();
        let c = IntegrationClient::from_name(name).unwrap();
        let artifact = &out["artifact"];
        let mut artifacts = Vec::new();
        if let Some(content) = artifact["snippet"].as_str() {
            artifacts.push(json!({"path":if c.install_kind()==InstallKind::Manual {None}else{Some(c.config_target())},"content":content}));
        }
        if c.is_terminal() {
            artifacts.push(json!({"path":RESOLVER_TARGET,"content":GRUND_OPEN_RESOLVER}));
        } else {
            artifacts.push(json!({"path":format!("{}/package.json",c.config_target()),"content":VSCODE_PACKAGE_JSON}));
            artifacts.push(json!({"path":format!("{}/extension.js",c.config_target()),"content":VSCODE_EXTENSION_JS}));
        }
        *out = json!({"mode":"artifact","client":descriptor(record),"artifacts":artifacts,
            "manual_steps":out["manual_steps"],"run_cautions":cautions});
    } else {
        *out = json!({"mode":"detection","detected":out["detected"],
            "clients":out["clients"].as_array().unwrap().iter().map(descriptor).collect::<Vec<_>>(),
            "run_cautions":cautions});
    }
}
