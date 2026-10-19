//! Real editor requests join equivalent numeric declaration/section citations
//! while returning their authored spans (§FS-config.3.2.8, §FS-lsp.1.3.1,
//! §FS-lsp.1.2.5).

#[path = "../../grund-core/tests/support/numeric_spelling.rs"]
mod fixture;
mod support;

use fixture::{NOTES, Repo};
use serde_json::{Value, json};
use support::*;

fn citation_location(repo: &Repo, token: &str) -> Value {
    let at = NOTES.find(token).unwrap();
    let start = NOTES[..at].encode_utf16().count();
    json!({
        "uri": file_uri(&repo.0.join("docs/notes.md")),
        "range": {
            "start": {"line":0,"character":start},
            "end": {"line":0,"character":start + token.encode_utf16().count()}
        }
    })
}

#[test]
fn equivalent_numeric_references_return_all_locations_and_hover_counts() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            let decision_uri = file_uri(&repo.0.join("docs/adr/decision.md"));
            let notes_uri = file_uri(&repo.0.join("docs/notes.md"));
            let (mut child, mut stdin, receiver) = start_server(&repo.0);
            let hover = hover_result(&mut stdin, &receiver, &mut child, 2, &decision_uri, 0, 5);
            assert!(
                hover["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .starts_with("`ADR-0001: Decision` — cited at 3 sites across 1 file"),
                "{hover}"
            );
            send_message(
                &mut stdin,
                json!({
                    "jsonrpc":"2.0", "id":3, "method":"textDocument/references",
                    "params":{
                        "textDocument":{"uri":decision_uri}, "position":{"line":0,"character":5},
                        "context":{"includeDeclaration":false}
                    }
                }),
            );
            let response = recv_response_or_panic(&receiver, &mut child, 3);
            let expected =
                ["§ADR-1", "§ADR-001.terms", "§ADR-0001"].map(|t| citation_location(&repo, t));
            assert_eq!(
                response["result"],
                json!(expected),
                "v{version}/{per_kind}: {response}"
            );

            let hover = hover_result(&mut stdin, &receiver, &mut child, 4, &decision_uri, 2, 6);
            assert!(
                hover["contents"]["value"]
                    .as_str()
                    .unwrap()
                    .starts_with("`terms: Terms` — cited at 1 site across 1 file"),
                "{hover}"
            );
            send_message(
                &mut stdin,
                json!({
                    "jsonrpc":"2.0", "id":5, "method":"textDocument/references",
                    "params":{
                        "textDocument":{"uri":decision_uri}, "position":{"line":2,"character":6},
                        "context":{"includeDeclaration":false}
                    }
                }),
            );
            let response = recv_response_or_panic(&receiver, &mut child, 5);
            assert_eq!(
                response["result"],
                json!([citation_location(&repo, "§ADR-001.terms")]),
                "{response}"
            );

            let at = NOTES.find("§ADR-001.terms").unwrap();
            send_message(
                &mut stdin,
                json!({
                    "jsonrpc":"2.0", "id":6, "method":"textDocument/references",
                    "params":{
                        "textDocument":{"uri":notes_uri},
                        "position":{"line":0,"character":NOTES[..at].encode_utf16().count() + 3},
                        "context":{"includeDeclaration":true}
                    }
                }),
            );
            let response = recv_response_or_panic(&receiver, &mut child, 6);
            assert_eq!(
                response["result"],
                json!([
                    {"uri":decision_uri,"range":{
                        "start":{"line":2,"character":3}, "end":{"line":2,"character":15}
                    }},
                    citation_location(&repo, "§ADR-001.terms")
                ]),
                "a citation's references retain its section scope: {response}"
            );

            send_message(
                &mut stdin,
                json!({"jsonrpc":"2.0","id":7,"method":"shutdown","params":null}),
            );
            recv_response_or_panic(&receiver, &mut child, 7);
            send_message(
                &mut stdin,
                json!({"jsonrpc":"2.0","method":"exit","params":null}),
            );
            drop(stdin);
            wait_for_exit(&mut child);
        }
    }
}
