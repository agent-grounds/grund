//! A citation hover answers from the snapshot the server holds
//! (§FS-lsp.responsiveness.3), and its body stays what `grund <ID> --toc`
//! prints (§FS-lsp.1.2.1, §REQ-one-answer.text.parity).
//!
//! The cost is counted rather than timed, as §FS-lsp.responsiveness asks: a
//! load of the workspace reports every input it is about to read through the
//! core's check-input observer before it reads it, so a hover that observes no
//! input loaded nothing the snapshot had not.

use super::*;
use grund_core::{CheckInput, ShowFormat, ShowMode, ShowOpts, show, with_check_input_observer};
use std::sync::{Arc, Mutex};

/// A workspace root with one member: a plain declaration, a numbered section, a
/// named section, a stub over an inline source declaration, and a citation of
/// each — the member's through its alias.
fn fixture(root: &Path) {
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\nproject_name = \"root\"\n\
         [reference]\nmarker = \"\u{a7}\"\nstrict = true\n\
         [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\ntitle = \"What\"\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\ntitle = \"How\"\n\
         [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"rs\"]\n\
         [workspace]\nmembers = [\"packages/app\"]\ninclude_root = true\n",
    );
    write(
        &root.join("docs/functional-spec/FS-alpha.md"),
        "# FS-alpha: Alpha\n\nThe lead.\n\n## 1. Detail\n\nMore.\n\n\
         ## goals: Goals\n\nGoals lead.\n\n### goals.speed: Speed\n\nFast.\n",
    );
    write(
        &root.join("docs/architecture/AR-router.md"),
        "# AR-router: [src/router.rs](../../src/router.rs)\n",
    );
    write(
        &root.join("src/router.rs"),
        "/// AR-router: Router\n///\n/// Realizes \u{a7}FS-alpha.1.\npub fn router() {}\n",
    );
    write(
        &root.join("docs/use.md"),
        "Plain \u{a7}FS-alpha.\n\
         Section \u{a7}FS-alpha.1.\n\
         Named \u{a7}FS-alpha.goals.speed.\n\
         Stub \u{a7}AR-router.\n\
         Member \u{a7}app/FS-app.\n",
    );
    let member = root.join("packages/app");
    write(
        &member.join("grund.toml"),
        "grund_config_version = 1\nproject_name = \"app\"\n\
         [reference]\nmarker = \"\u{a7}\"\nstrict = true\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\ntitle = \"What\"\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    write(
        &member.join("docs/functional-spec/FS-app.md"),
        "# FS-app: App\n\nThe member's lead.\n",
    );
}

/// The lines of `docs/use.md` and the ID each one cites.
const CASES: [(u32, &str); 5] = [
    (0, "FS-alpha"),
    (1, "FS-alpha.1"),
    (2, "FS-alpha.goals.speed"),
    (3, "AR-router"),
    (4, "app/FS-app"),
];

fn server_over(root: &Path) -> (Server, Connection) {
    let (connection, client) = Connection::memory();
    let server = Server::new(
        connection,
        [(Url::from_directory_path(root).unwrap(), root.to_path_buf())]
            .into_iter()
            .collect(),
        true,
    )
    .unwrap();
    (server, client)
}

/// The hover on the citation at `line` of `docs/use.md`: its Markdown body,
/// and every workspace input the core observed while answering it.
fn hover(
    server: &mut Server,
    client: &Connection,
    root: &Path,
    line: u32,
) -> (String, Vec<CheckInput>) {
    let uri = Url::from_file_path(root.join("docs/use.md")).unwrap();
    let column = server
        .document_text(&uri)
        .and_then(|text| text.lines().nth(line as usize).map(str::to_string))
        .and_then(|text| {
            text.find('\u{a7}')
                .map(|byte| byte_to_utf16(&text, byte) + 1)
        })
        .expect("a citation on the line");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    with_check_input_observer(
        Some(Arc::new(move |input| {
            sink.lock().unwrap().push(input);
            true
        })),
        || {
            server.handle_request(Request::new(
                1.into(),
                "textDocument/hover".into(),
                json!({
                    "textDocument": {"uri": uri},
                    "position": {"line": line, "character": column}
                }),
            ))
        },
    )
    .expect("hover");
    let Message::Response(response) = client.receiver.recv().unwrap() else {
        panic!("response")
    };
    assert!(response.error.is_none(), "{:?}", response.error);
    let body = response
        .result
        .as_ref()
        .and_then(|result| result["contents"]["value"].as_str())
        .unwrap_or_else(|| panic!("hover body on line {line}: {:?}", response.result))
        .to_string();
    let observed = observed.lock().unwrap().clone();
    (body, observed)
}

/// What the hover must carry for `id`: the body `grund <ID> --toc` prints over
/// `root` as it is on disk, through the transport's own two decorations —
/// citation links and the kind line (§FS-lsp.1.2.1, §FS-lsp.1.2.8).
fn cli_answer(server: &Server, root: &Path, id: &str) -> String {
    let output = show(
        id,
        ShowOpts {
            path: root.to_path_buf(),
            section: None,
            mode: ShowMode::Toc,
            format: ShowFormat::Markdown,
        },
    )
    .unwrap_or_else(|err| panic!("grund {id} --toc: {err:#}"));
    let project = server.projects.first().expect("the root project");
    let citation = project
        .snapshot
        .citations
        .iter()
        .find(|citation| citation.text == format!("\u{a7}{id}"))
        .unwrap_or_else(|| panic!("the snapshot records the citation of {id}"));
    lsp_hover_with_kind_title(
        &linkify_hover_body(&output.body, &project.snapshot.citations),
        project
            .kind_titles
            .get(&citation.declaration_query_id)
            .map(String::as_str),
    )
}

/// §FS-lsp.responsiveness.3: once a snapshot exists, a citation hover loads no
/// workspace and observes no input the snapshot had not already read. Today it
/// re-runs `show` per hover, which reloads the whole workspace.
#[test]
fn a_citation_hover_loads_no_workspace_once_a_snapshot_exists() {
    let root = test_root("hover_from_snapshot_cost");
    fixture(&root);
    let (mut server, client) = server_over(&root);
    for (line, id) in CASES {
        let (_, observed) = hover(&mut server, &client, &root, line);
        assert!(
            observed.is_empty(),
            "the hover on {id} observed {} workspace inputs, {:?}…: it reloaded the workspace \
             instead of answering from the snapshot (\u{a7}FS-lsp.responsiveness.3)",
            observed.len(),
            observed
                .iter()
                .take(3)
                .map(|input| &input.path)
                .collect::<Vec<_>>()
        );
    }
}

/// §FS-lsp.1.2.1, §REQ-one-answer.text.parity: the hover body is, byte for byte,
/// what `grund <ID> --toc` prints, for a plain ID, a numbered section, a named
/// section, a stub and a member's ID through its alias. This passes today; it is
/// the guard that lets §FS-lsp.responsiveness.3 change where the body comes from
/// without changing a byte of it.
#[test]
fn a_citation_hover_is_what_show_toc_prints() {
    let root = test_root("hover_from_snapshot_parity");
    fixture(&root);
    let (mut server, client) = server_over(&root);
    for (line, id) in CASES {
        let (body, _) = hover(&mut server, &client, &root, line);
        assert_eq!(body, cli_answer(&server, &root, id), "hover on {id}");
    }
}

/// §FS-lsp.4.1 with §FS-lsp.responsiveness.3: an unsaved edit to the cited body
/// is in the hover, the same bytes `grund <ID> --toc` prints once it is saved.
/// A snapshot-answered hover must take the body from the snapshot's buffers,
/// never from disk. This passes today.
#[test]
fn a_citation_hover_shows_the_unsaved_body_as_saved() {
    let root = test_root("hover_from_snapshot_unsaved");
    fixture(&root);
    let (mut server, client) = server_over(&root);
    let spec = root.join("docs/functional-spec/FS-alpha.md");
    let edited = "# FS-alpha: Alpha\n\nAn unsaved lead.\n\n## 1. Detail\n\nEdited.\n\n\
                  ## goals: Goals\n\nGoals lead.\n\n### goals.speed: Speed\n\nFaster.\n";
    server
        .open_docs
        .insert(Url::from_file_path(&spec).unwrap(), edited.to_string());
    server.refresh();
    let unsaved: Vec<_> = CASES[..3]
        .iter()
        .map(|&(line, _)| hover(&mut server, &client, &root, line).0)
        .collect();
    assert!(unsaved[0].contains("An unsaved lead."), "{:?}", unsaved[0]);

    write(&spec, edited);
    for (body, (_, id)) in unsaved.iter().zip(&CASES[..3]) {
        assert_eq!(
            *body,
            cli_answer(&server, &root, id),
            "unsaved hover on {id}"
        );
    }
}
