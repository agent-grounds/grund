//! The editor's rows of the mount proof (§REQ-remote-equivalence.4, §FS-imports.editor): a
//! document under an import, in a session over the consumer alone, is answered as a session
//! over the dependency's own folder answers it, for every capability the server advertises.
//! The edits and diagnostics a read-only import withholds are the one difference, and each
//! row says which (§FS-lsp.2.2.3). One row per advertised capability, so a capability added
//! without a row fails; the command-line rows are `crates/grund-cli/tests/mount_operations.rs`.

#[path = "binaries.rs"]
mod binaries;
#[path = "lsp_import_session.rs"]
mod lsp_import_session;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use lsp_import_session::{Composition, DEPENDENCY, MAIN, Session};
use serde_json::{Value, json};

type Row = fn(&Composition, &mut Session<'_>, &mut Session<'_>);

/// Each capability the server advertises, and its row.
const ROWS: &[(&str, Row)] = &[
    ("completionProvider", completion),
    ("definitionProvider", definition),
    ("documentHighlightProvider", highlights),
    ("documentLinkProvider", links),
    ("documentOnTypeFormattingProvider", on_type),
    ("hoverProvider", hover),
    ("referencesProvider", references),
    ("textDocumentSync", diagnostics),
    ("workspace", folders),
];

/// `<§>FS-security-providers` on line 0 of `MAIN`, cited from inside the dependency.
fn at(composition: &Composition) -> Value {
    json!({"textDocument": {"uri": composition.uri(MAIN)}, "position": {"line": 0, "character": 25}})
}

fn same(imported: &mut Session, native: &mut Session, method: &str, params: Value) {
    let answer = imported.request(method, params.clone());
    assert!(
        !answer.is_null(),
        "{method} answers nothing through the import"
    );
    assert_eq!(answer, native.request(method, params), "{method}");
}

fn hover(composition: &Composition, imported: &mut Session, native: &mut Session) {
    same(imported, native, "textDocument/hover", at(composition));
}

fn definition(composition: &Composition, imported: &mut Session, native: &mut Session) {
    same(imported, native, "textDocument/definition", at(composition));
}

fn references(composition: &Composition, imported: &mut Session, native: &mut Session) {
    let mut params = at(composition);
    params["context"] = json!({"includeDeclaration": true});
    same(imported, native, "textDocument/references", params);
}

fn highlights(composition: &Composition, imported: &mut Session, native: &mut Session) {
    same(
        imported,
        native,
        "textDocument/documentHighlight",
        at(composition),
    );
}

fn links(composition: &Composition, imported: &mut Session, native: &mut Session) {
    let params = json!({"textDocument": {"uri": composition.uri(MAIN)}});
    same(imported, native, "textDocument/documentLink", params);
}

/// How many items `session` answers `method` with, after `typed` on line 0 of `MAIN`.
fn offered(session: &mut Session, method: &str, typed: &str, params: &Value) -> usize {
    session.open(MAIN, &format!("{typed}\nclass Main {{}}\n"));
    let answer = session.request(method, params.clone());
    let items = if answer["items"].is_array() {
        &answer["items"]
    } else {
        &answer
    };
    items.as_array().map_or(0, Vec::len)
}

/// Withheld: the import is read-only, so it is offered no completion edit.
fn completion(composition: &Composition, imported: &mut Session, native: &mut Session) {
    let typed = "// see \u{a7}FS-sec";
    let params = json!({"textDocument": {"uri": composition.uri(MAIN)},
        "position": {"line": 0, "character": typed.chars().count()}});
    let method = "textDocument/completion";
    let imported = offered(imported, method, typed, &params);
    assert_eq!([imported, offered(native, method, typed, &params)], [0, 1]);
}

/// Withheld: the import is offered no on-type edit.
fn on_type(composition: &Composition, imported: &mut Session, native: &mut Session) {
    let typed = "// see $$FS-security-providers";
    let params = json!({"textDocument": {"uri": composition.uri(MAIN)},
        "position": {"line": 0, "character": typed.len()}, "ch": "s",
        "options": {"tabSize": 4, "insertSpaces": true}});
    let method = "textDocument/onTypeFormatting";
    let imported = offered(imported, method, typed, &params);
    assert_eq!([imported, offered(native, method, typed, &params)], [0, 1]);
}

/// Withheld: the import's findings are the dependency's, so the consumer publishes none.
fn diagnostics(_: &Composition, imported: &mut Session, native: &mut Session) {
    assert_eq!(imported.diagnostics_on(MAIN), Vec::<Value>::new());
    let codes = native
        .diagnostics_on(MAIN)
        .iter()
        .map(|d| d["code"].clone())
        .collect::<Vec<_>>();
    assert_eq!(codes, [json!("dangling")]);
}

/// The dependency added as a folder owns its files, and publishes as its own session does.
fn folders(_: &Composition, imported: &mut Session, native: &mut Session) {
    imported.folders(&[DEPENDENCY], &[]);
    assert_eq!(imported.diagnostics_on(MAIN), native.diagnostics_on(MAIN));
}

#[test]
fn every_advertised_capability_has_an_inventory_row() {
    let composition = Composition::new("mount-inventory");
    let session = Session::start(&composition, &["enterprise"], true);
    let advertised = session.capabilities.as_object().expect("capabilities");
    let advertised = advertised.keys().map(String::as_str).collect::<Vec<_>>();
    let rows = ROWS
        .iter()
        .map(|(capability, _)| *capability)
        .collect::<Vec<_>>();
    assert_eq!(
        advertised, rows,
        "a capability without a row, or a row without one"
    );
    session.shutdown();
}

#[test]
fn an_imported_document_answers_as_the_dependency_session_does() {
    for (capability, row) in ROWS {
        let composition = Composition::new(&format!("mount-{capability}"));
        let main = std::fs::read_to_string(composition.path(MAIN)).expect("Main.java");
        let mut imported = Session::start(&composition, &["enterprise"], true);
        let mut native = Session::start(&composition, &[DEPENDENCY], true);
        imported.open(MAIN, &main);
        native.open(MAIN, &main);
        row(&composition, &mut imported, &mut native);
        imported.shutdown();
        native.shutdown();
    }
}
