//! The existing manifest argv grammar, with isolated path expansion
//! (§FS-examples.5.3, §FS-examples.5.4, §REQ-one-answer.proof.matrix).

use super::agreement_oracles::text;
use serde_json::Value;

include!("case_words.rs");

pub fn command(context: &Value) -> (Option<String>, Vec<String>) {
    let manifests = &context["manifests"];
    let external = manifests.get("command.external");
    let words = if let Some(external) = external {
        assert!(
            manifests.get("command.args").is_none(),
            "mutually exclusive manifests"
        );
        let words: Vec<String> =
            serde_json::from_str(external.as_str().unwrap()).expect("external JSON argv");
        assert!(
            words.first().is_some_and(|w| !w.is_empty()),
            "external executable"
        );
        words
    } else {
        split_command(
            &text(context, "case"),
            manifests["command.args"]
                .as_str()
                .unwrap_or(DEFAULT_COMMAND),
        )
    };
    let root = text(context, "base");
    let mut expanded: Vec<String> = words
        .into_iter()
        .map(|word| {
            if external.is_some() && word == "{grund}" {
                return env!("CARGO_BIN_EXE_grund").into();
            }
            for token in ["{repo}", "{repo_copy}"] {
                if word == token || word.starts_with(&format!("{token}/")) {
                    return format!("{root}{}", &word[token.len()..]);
                }
            }
            word
        })
        .collect();
    if external.is_some() {
        (Some(expanded.remove(0)), expanded)
    } else {
        (None, expanded)
    }
}
