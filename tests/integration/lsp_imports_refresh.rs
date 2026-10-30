//! The editor follows an import's tree as the command line reads it, live
//! (§FS-imports.editor, §AR-lsp.2.1): a dependency edit on disk, its config, an unsaved
//! buffer under any spelling, a file created and deleted, the target disappearing, a
//! config that will not load, and folders added and removed. A document under the
//! import is answered read-only by the import's own project, under one consumer or
//! two (§FS-lsp.2.2.3), and is offered no edit.

#[path = "binaries.rs"]
mod binaries;
#[path = "lsp_import_session.rs"]
mod lsp_import_session;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use lsp_import_session::{
    Composition, DECLARATION, DEPENDENCY, DEPENDENCY_CONFIG, EXAMPLE, MAIN, Session, declaration,
};
use serde_json::{Value, json};

const TITLE: &str = "Registered at build time";
const CONFIG: &str = "graal/substratevm/grund.toml";
const CHANGED: u8 = 2;

/// A session over the consumer's folder alone, with its citation open.
fn consumer<'a>(composition: &'a Composition, watchers: bool) -> Session<'a> {
    let mut session = Session::start(composition, &["enterprise"], watchers);
    session.open("enterprise/src/Example.java", EXAMPLE);
    assert!(
        session
            .hover_until(|hover| hover.is_some())
            .is_some_and(|hover| hover.contains(TITLE))
    );
    session
}

fn has(hover: Option<&str>, text: &str) -> bool {
    hover.is_some_and(|hover| hover.contains(text))
}

fn codes(diagnostics: &[Value]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic["code"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// The edits `onTypeFormatting` offers after `$$FS-security-providers` on line 0 of `relative`.
fn on_type(session: &mut Session, composition: &Composition, relative: &str) -> Value {
    let line = "// see $$FS-security-providers";
    session.open(relative, &format!("{line}\nclass Main {{}}\n"));
    session.request(
        "textDocument/onTypeFormatting",
        json!({"textDocument": {"uri": composition.uri(relative)},
            "position": {"line": 0, "character": line.len()}, "ch": "s",
            "options": {"tabSize": 4, "insertSpaces": true}}),
    )
}

fn no_edit(edits: &Value) -> bool {
    edits.is_null() || edits.as_array().is_some_and(Vec::is_empty)
}

#[test]
fn the_server_registers_a_watcher_on_the_dependency_and_follows_its_events() {
    let composition = Composition::new("registered");
    let mut session = consumer(&composition, true);
    let registrations = session.registrations();
    let watchers = registrations
        .iter()
        .flat_map(|request| {
            request["params"]["registrations"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .filter(|registration| registration["method"] == "workspace/didChangeWatchedFiles")
        .flat_map(|registration| {
            registration["registerOptions"]["watchers"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let dependency = composition.uri(DEPENDENCY);
    assert!(
        watchers
            .iter()
            .any(|watcher| watcher["globPattern"]["baseUri"]
                .as_str()
                .is_some_and(|base| base.trim_end_matches('/') == dependency)),
        "no watcher on the dependency: {registrations:?}"
    );
    composition.write(DECLARATION, &declaration("DISK TITLE"));
    session.watched(&[(DECLARATION, CHANGED)]);
    assert!(has(
        session
            .hover_until(|hover| has(hover, "DISK TITLE"))
            .as_deref(),
        "DISK TITLE"
    ));
    session.shutdown();
}

#[test]
fn without_client_watchers_the_server_watches_the_dependency_itself() {
    let composition = Composition::new("native");
    let mut session = consumer(&composition, false);
    assert!(
        session.registrations().is_empty(),
        "{:?}",
        session.registrations()
    );
    composition.write(DECLARATION, &declaration("DISK TITLE"));
    let hover = session.hover_until(|hover| has(hover, "DISK TITLE"));
    assert!(
        has(hover.as_deref(), "DISK TITLE"),
        "a silent disk edit stays stale: {hover:?}"
    );
    session.shutdown();
}

#[test]
fn a_dependency_config_edit_is_followed() {
    let composition = Composition::new("config-edit");
    let mut session = consumer(&composition, true);
    composition.write(
        CONFIG,
        &DEPENDENCY_CONFIG.replace("\"native-image\"", "\"other\""),
    );
    session.watched(&[(CONFIG, CHANGED)]);
    let diagnostics = session.diagnostics_on("enterprise/grund.toml");
    let refusal = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["severity"] == 1);
    let refusal = refusal.unwrap_or_else(|| panic!("no refusal: {diagnostics:?}"));
    assert!(
        refusal["message"]
            .as_str()
            .unwrap_or_default()
            .contains("declares `other`"),
        "{refusal}"
    );
    assert!(
        has(session.hover_until(|_| true).as_deref(), TITLE),
        "the last good scan answers"
    );
    composition.write(CONFIG, DEPENDENCY_CONFIG);
    session.watched(&[(CONFIG, CHANGED)]);
    assert_eq!(
        codes(&session.diagnostics_on("enterprise/grund.toml")),
        ["import-live"]
    );
    session.shutdown();
}

#[test]
fn an_unsaved_dependency_buffer_reaches_the_consumer() {
    let composition = Composition::new("unsaved");
    let mut session = consumer(&composition, true);
    session.open(DECLARATION, &declaration("UNSAVED TITLE"));
    assert!(has(
        session
            .hover_until(|hover| has(hover, "UNSAVED TITLE"))
            .as_deref(),
        "UNSAVED TITLE"
    ));
    session.shutdown();
}

#[cfg(unix)]
#[test]
fn a_symlinked_spelling_of_the_dependency_is_one_buffer() {
    let composition = Composition::new("symlinked");
    std::os::unix::fs::symlink(composition.path("graal"), composition.path("graal-link"))
        .expect("symlink");
    let mut session = consumer(&composition, true);
    let linked =
        composition.uri("graal-link/substratevm/docs/functional-spec/FS-security-providers.md");
    session.open_uri(&linked, &declaration("UNSAVED VIA LINK"));
    let hover = session.hover_until(|hover| has(hover, "UNSAVED VIA LINK"));
    assert!(has(hover.as_deref(), "UNSAVED VIA LINK"), "{hover:?}");
    session.shutdown();
}

#[test]
fn a_file_created_and_deleted_in_the_dependency_is_followed() {
    let composition = Composition::new("created");
    let mut session = consumer(&composition, true);
    let citing = format!("{EXAMPLE}// and \u{a7}native-image/FS-new\n");
    session.open("enterprise/src/Example.java", &citing);
    let dangling = |session: &mut Session| {
        codes(&session.diagnostics_on("enterprise/src/Example.java"))
            .contains(&"dangling".to_string())
    };
    assert!(dangling(&mut session));
    let created = "graal/substratevm/docs/functional-spec/FS-new.md";
    composition.write(created, "# FS-new: New\n");
    session.watched(&[(created, 1)]);
    assert!(!dangling(&mut session), "the created declaration resolves");
    std::fs::remove_file(composition.path(created)).expect("delete");
    session.watched(&[(created, 3)]);
    assert!(
        dangling(&mut session),
        "the deleted declaration dangles again"
    );
    session.shutdown();
}

#[test]
fn a_dependency_that_disappears_rebuilds_the_consumer_without_it() {
    let composition = Composition::new("gone");
    let mut session = consumer(&composition, true);
    std::fs::rename(composition.path("graal"), composition.path("graal-gone")).expect("move away");
    session.watched(&[(DEPENDENCY, 3)]);
    assert_eq!(session.hover_until(|hover| hover.is_none()), None);
    let diagnostics = session.diagnostics_on("enterprise/grund.toml");
    let missing = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"] == "import-missing");
    assert_eq!(
        missing.map(|diagnostic| diagnostic["range"]["start"]["line"].clone()),
        Some(json!(3))
    );
    session.shutdown();
}

#[test]
fn a_malformed_dependency_config_keeps_the_last_good_scan() {
    let composition = Composition::new("malformed");
    let mut session = consumer(&composition, true);
    composition.write(
        CONFIG,
        &DEPENDENCY_CONFIG.replace("project_name", "strcit = 1\nproject_name"),
    );
    session.watched(&[(CONFIG, CHANGED)]);
    let diagnostics = session.diagnostics_on("enterprise/grund.toml");
    let refusal = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["severity"] == 1);
    let refusal =
        refusal.unwrap_or_else(|| panic!("no refusal on the import line: {diagnostics:?}"));
    let message = refusal["message"].as_str().unwrap_or_default();
    assert_eq!(refusal["range"]["start"]["line"], 3, "{refusal}");
    for expected in [
        "unknown key `strcit`",
        "note: imported as `native-image` from grund.toml:4",
        "last good scan",
    ] {
        assert!(
            message.contains(expected),
            "`{expected}` is not in {message:?}"
        );
    }
    assert!(
        has(session.hover_until(|_| true).as_deref(), TITLE),
        "the last good scan answers"
    );
    composition.write(CONFIG, DEPENDENCY_CONFIG);
    session.watched(&[(CONFIG, CHANGED)]);
    assert_eq!(
        codes(&session.diagnostics_on("enterprise/grund.toml")),
        ["import-live"]
    );
    session.shutdown();
}

/// §FS-lsp.2.2.3: the import's own project answers, whichever consumers hold it and in
/// whichever order; it publishes nothing and offers no edit.
#[test]
fn an_import_owned_document_answers_read_only_under_one_and_two_consumers() {
    for folders in [
        &["enterprise"][..],
        &["enterprise", "enterprise2"],
        &["enterprise2", "enterprise"],
    ] {
        let composition = Composition::new("owned");
        let mut session = Session::start(&composition, folders, true);
        let main = std::fs::read_to_string(composition.path(MAIN)).expect("Main.java");
        session.open(MAIN, &main);
        // `<§>FS-security-providers` starts at UTF-16 column 19 of line 0.
        let hover = session.hover(MAIN, 0, 25);
        assert!(has(hover.as_deref(), TITLE), "{folders:?}: {hover:?}");
        assert_eq!(
            session.diagnostics_on(MAIN),
            Vec::<Value>::new(),
            "{folders:?}"
        );
        assert!(
            no_edit(&on_type(&mut session, &composition, MAIN)),
            "{folders:?}"
        );
        session.shutdown();
    }
}

/// §FS-imports.editor: opened as its own folder, the dependency owns its files and edits
/// them normally; removed again, it is the import's.
#[test]
fn folders_added_and_removed_move_the_dependency_between_owners() {
    let composition = Composition::new("folders");
    let mut session = consumer(&composition, true);
    let main = std::fs::read_to_string(composition.path(MAIN)).expect("Main.java");
    session.open(MAIN, &main);
    assert_eq!(session.diagnostics_on(MAIN), Vec::<Value>::new());
    session.folders(&[DEPENDENCY], &[]);
    assert_eq!(
        codes(&session.diagnostics_on(MAIN)),
        ["dangling"],
        "its own folder reports it"
    );
    let edits = on_type(&mut session, &composition, MAIN);
    assert_eq!(
        edits[0]["newText"], "\u{a7}",
        "its own folder edits it: {edits}"
    );
    session.open(MAIN, &main);
    session.folders(&[], &[DEPENDENCY]);
    assert_eq!(
        session.diagnostics_on(MAIN),
        Vec::<Value>::new(),
        "the import's again"
    );
    assert!(has(session.hover(MAIN, 0, 25).as_deref(), TITLE));
    session.shutdown();
}

/// §FS-imports.editor: no edit in an imported document, and the usual one in the same
/// document once the dependency is a folder of the session.
#[test]
fn on_type_formatting_offers_no_edit_in_an_imported_document() {
    let composition = Composition::new("on-type");
    let mut session = consumer(&composition, true);
    assert!(no_edit(&on_type(&mut session, &composition, MAIN)));
    session.shutdown();
    let mut session = Session::start(&composition, &[DEPENDENCY], true);
    let edits = on_type(&mut session, &composition, MAIN);
    assert_eq!(
        edits[0]["newText"], "\u{a7}",
        "the dependency's own folder: {edits}"
    );
    session.shutdown();
}
