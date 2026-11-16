//! The rebuild worker held closed (§AR-lsp.2.1.2): a typing request is answered while no rebuild
//! can finish when the edits before it left the declarations standing, and waits for the rebuilt
//! catalog when they did not (§FS-lsp.responsiveness.4); edits merge into one rebuild, a snapshot
//! request is answered against the text it was sent with (§FS-lsp.responsiveness.3), and a held
//! request the client cancels is answered cancelled. Each case drives the real event loop through
//! the typing harness, its worker behind a gate it releases rebuild by rebuild.

use super::held_client::*;
use super::hover_snapshot::{cross_folder_fixture, declaring, server_over_folders};
use super::typing::{AFTER_FS_00, drive, fixture, note, server_over, uri};
use super::*;

/// Case 1, §FS-lsp.responsiveness.4: a key typed into prose is completed while the rebuild of
/// its edit is held, with what a full rebuild offers.
#[test]
fn a_completion_while_the_rebuild_is_held_offers_a_full_rebuilds_items() {
    let root = test_root("typing_held_completion");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let typed = note("see $$FS-00");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (path, text) = (notes.clone(), typed.clone());
    let ((held, after), _) = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&path, &note("see $$FS-0"))]);
        client.change(&path, &text, 2);
        hold.started();
        let held = client.completion(&path, &text, 2);
        let events = hold.rest();
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Rebuilding::Finished { .. })),
            "a rebuild completed before the completion was answered: {events:?}"
        );
        hold.open();
        client.settle(&path);
        (held, client.completion(&path, &text, 2))
    });
    let at = completion_params(&notes, &typed, 2);
    let fresh = labels(
        &fresh(
            &[&root],
            &[(&notes, &typed)],
            "textDocument/completion",
            &[at],
        )[0],
    );
    for mut offered in [held, after] {
        offered.sort();
        assert_eq!(offered, AFTER_FS_00, "\u{a7}FS-lsp.responsiveness.4");
        assert_eq!(
            offered, fresh,
            "a fresh server answers that buffer otherwise"
        );
    }
}

/// Case 2, §FS-lsp.responsiveness.4: two keys typed while held are each completed for their own
/// text, each edit's range ending where that key left the line.
#[test]
fn back_to_back_keys_are_each_answered_for_their_own_text() {
    let root = test_root("typing_held_back_to_back");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let path = notes.clone();
    let (answers, _) = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&path, &note("See "))]);
        let mut ids = Vec::new();
        for (version, line) in [(2, "See $$F"), (3, "See $$FS")] {
            let text = note(line);
            client.change(&path, &text, version);
            ids.push(send(
                client,
                "textDocument/completion",
                completion_params(&path, &text, 2),
            ));
        }
        let answers: Vec<Value> = ids
            .into_iter()
            .map(|id| result(answer(client, id, &mut Vec::new())))
            .collect();
        hold.open();
        answers
    });
    for (offered, typed) in answers.iter().zip(["See $$F", "See $$FS"]) {
        assert_eq!(
            labels(offered),
            ["FS-001-intro", "FS-002-login", "FS-020-logout"],
            "the completion after `{typed}`"
        );
        let end = typed.encode_utf16().count() as u64;
        for item in offered["items"].as_array().unwrap() {
            let range = &item["textEdit"]["range"];
            assert_eq!(range["end"]["line"], 2, "{item}");
            assert_eq!(
                range["end"]["character"].as_u64(),
                Some(end),
                "the edit offered after `{typed}` must end where that key left the line: {item}"
            );
        }
    }
}

/// Case 3, §AR-lsp.2.1.2: five edits made while one rebuild is held merge into exactly one
/// further rebuild over the latest text, and no version between is published (§FS-lsp.1.1).
#[test]
fn five_held_edits_yield_exactly_one_further_rebuild_over_the_latest_text() {
    let root = test_root("typing_held_merge");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let text = |version: i32| note(&format!("Version {version}, \u{a7}FS-9{version:02}-gone."));
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let path = notes.clone();
    let ((first, further, published), _) = drive(server, connection, move |client| {
        client.open(&path, &text(1), 1);
        let first = hold.started();
        for version in 2..=6 {
            client.change(&path, &text(version), version);
        }
        // Answered from the judgement, so only once the loop has taken every edit before it:
        // they merge before the first rebuild may finish (§FS-lsp.responsiveness.4).
        client.completion(&path, &text(6), 2);
        hold.open();
        client.settle(&path);
        let further: Vec<BTreeMap<Url, String>> = hold
            .rest()
            .into_iter()
            .filter_map(|event| match event {
                Rebuilding::Started { docs, .. } => Some(docs),
                Rebuilding::Finished { .. } => None,
            })
            .collect();
        (first, further, client.published.take())
    });
    let notes_uri = uri(&notes);
    assert_eq!(first.1.get(&notes_uri), Some(&text(1)));
    assert_eq!(
        further.len(),
        1,
        "one further rebuild, not {}",
        further.len()
    );
    assert_eq!(
        further[0].get(&notes_uri),
        Some(&text(6)),
        "over the latest text"
    );
    let versions: BTreeSet<Option<i32>> = published
        .iter()
        .filter(|publish| publish.uri == notes_uri)
        .map(|publish| publish.version)
        .collect();
    assert!(versions.contains(&Some(6)), "{versions:?}");
    assert!(
        versions.is_subset(&BTreeSet::from([Some(1), Some(6)])),
        "notes was published for {versions:?}: only versions 1 and 6 were rebuilt"
    );
}

/// Case 4, §FS-lsp.responsiveness.3: a hover held behind its edit's rebuild is answered from the
/// snapshot of that edit, though a later edit removed what it points at before the release.
#[test]
fn a_hover_sent_before_a_further_edit_is_answered_against_its_own_version() {
    let root = test_root("typing_held_hover_version");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let cited = note("See \u{a7}FS-001-intro.");
    let at = position(&notes, 2, 6);
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (path, text, params) = (notes.clone(), cited.clone(), at.clone());
    let ((hover, rebuilt), _) = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&path, &note("Plain prose."))]);
        client.change(&path, &text, 2);
        let id = send(client, "textDocument/hover", params);
        client.change(&path, &note("See nothing."), 3);
        hold.taken(client.sent.get());
        unanswered(client, &[id], "a hover waits for its edit's snapshot");
        hold.open();
        let hover = result(answer(client, id, &mut Vec::new()));
        client.settle(&path);
        let rebuilt: Vec<String> = hold
            .rest()
            .into_iter()
            .filter_map(|event| match event {
                Rebuilding::Started { mut docs, .. } => docs.remove(&uri(&path)),
                Rebuilding::Finished { .. } => None,
            })
            .collect();
        (hover, rebuilt)
    });
    assert_ne!(
        hover,
        Value::Null,
        "the hover on v2's citation found nothing"
    );
    let fresh = fresh(&[&root], &[(&notes, &cited)], "textDocument/hover", &[at]);
    assert_eq!(hover, fresh[0], "a fresh server hovers v2 otherwise");
    assert_eq!(
        rebuilt,
        [cited, note("See nothing.")],
        "rebuilds for v2, sealed, then v3"
    );
}

/// Case 5, §FS-lsp.responsiveness.4: an edit that renames a declaration holds the completion
/// after it until the rebuilt catalog holds the new name.
#[test]
fn a_completion_after_a_heading_edit_waits_for_the_rebuilt_catalog() {
    let root = test_root("typing_held_heading_edit");
    fixture(&root);
    let (notes, spec) = (root.join("docs/notes.md"), root.join(SPEC));
    let typed = note("see $$FS-00");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let offered = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&notes, &typed), (&spec, LOGIN)]);
        client.change(&spec, SIGNIN, 2);
        let id = send(
            client,
            "textDocument/completion",
            completion_params(&notes, &typed, 2),
        );
        hold.taken(client.sent.get());
        unanswered(client, &[id], "the edit renamed a declaration");
        hold.open();
        labels(&result(answer(client, id, &mut Vec::new())))
    })
    .0;
    assert_eq!(offered, ["FS-001-intro", "FS-002-signin"]);
}

/// Case 6, §AR-lsp.2.1.2: a held hover the client cancels is answered `RequestCancelled` before
/// any rebuild is let through, and never answered again.
#[test]
fn a_held_hover_the_client_cancels_is_answered_request_cancelled() {
    let root = test_root("typing_held_cancel");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (cancelled, again) = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&notes, &note("Plain prose."))]);
        client.change(&notes, &note("Plain prose, edited."), 2);
        let id = send(client, "textDocument/hover", position(&notes, 2, 0));
        client.notify("$/cancelRequest", json!({"id": id}));
        let mut strays = Vec::new();
        let cancelled = answer(client, id, &mut strays);
        hold.open();
        let settle = send(client, "textDocument/hover", position(&notes, 0, 0));
        answer(client, settle, &mut strays);
        while let Some(response) = next_response(client, QUIET) {
            strays.push(response);
        }
        let again = strays.iter().filter(|stray| stray.id == id.into()).count();
        (cancelled, again)
    })
    .0;
    let error = cancelled.error.expect("the cancelled hover's error");
    assert_eq!(error.code, -32800, "RequestCancelled: {error:?}");
    assert_eq!(again, 0, "the cancelled hover was answered a second time");
}

/// Case 7, §FS-lsp.responsiveness.4 (R4): a heading renamed and renamed back before the first
/// rebuild lands is judged again when that stale catalog installs, so the name between is never
/// offered.
#[test]
fn a_heading_renamed_back_before_its_rebuild_lands_never_offers_the_name_between() {
    let root = test_root("typing_held_rename_back");
    fixture(&root);
    let (notes, spec) = (root.join("docs/notes.md"), root.join(SPEC));
    let typed = note("see $$FS-00");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (answers, _) = drive(server, connection, move |client| {
        opened_then_held(client, &hold, &[(&notes, &typed), (&spec, LOGIN)]);
        client.change(&spec, SIGNIN, 2);
        hold.started();
        client.change(&spec, LOGIN, 3);
        hold.taken(client.sent.get());
        hold.release(1);
        hold.finished();
        let (_, docs) = hold.started();
        assert_eq!(docs.get(&uri(&spec)).map(String::as_str), Some(LOGIN));
        let id = send(
            client,
            "textDocument/completion",
            completion_params(&notes, &typed, 2),
        );
        let mut responses = Vec::new();
        while let Some(response) = next_response(client, QUIET) {
            responses.push(response);
        }
        hold.open();
        if !responses.iter().any(|response| response.id == id.into()) {
            let mut strays = Vec::new();
            responses.push(answer(client, id, &mut strays));
            responses.extend(strays);
        }
        client.settle(&notes);
        responses
            .into_iter()
            .filter(|response| response.id == id.into())
            .map(result)
            .collect::<Vec<_>>()
    });
    assert_eq!(answers.len(), 1, "{answers:?}");
    assert_eq!(labels(&answers[0]), AFTER_FS_00, "never FS-002-signin");
}

/// §FS-lsp.responsiveness.4 (R4): a key typed after a heading is renamed back, and before the
/// stale rebuild of the rename lands, is answered from the catalog the text stands in; that
/// stale catalog's install judges the edit again, so the `$$` typed next waits for the rebuild of
/// the name back, and no answer ever offers the name between.
#[test]
fn a_key_typed_before_the_stale_rebuild_lands_is_judged_again_when_it_installs() {
    let root = test_root("typing_held_rename_back_early_key");
    fixture(&root);
    let (notes, spec) = (root.join("docs/notes.md"), root.join(SPEC));
    let (typed, dollars) = (note("see $$FS-00"), note("see $$"));
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (paths, texts) = (
        (notes.clone(), spec.clone()),
        (typed.clone(), dollars.clone()),
    );
    let ((early, late), _) = drive(server, connection, move |client| {
        let ((notes, spec), (typed, dollars)) = (paths, texts);
        opened_then_held(client, &hold, &[(&notes, &typed), (&spec, LOGIN)]);
        client.change(&spec, SIGNIN, 2);
        hold.started();
        client.change(&spec, LOGIN, 3);
        let early = send(
            client,
            "textDocument/completion",
            completion_params(&notes, &typed, 2),
        );
        let early = result(answer(client, early, &mut Vec::new()));
        hold.release(1);
        hold.finished();
        // The rename back's rebuild starts once the stale catalog is installed.
        hold.started();
        client.change(&notes, &dollars, 2);
        let late = send(
            client,
            "textDocument/completion",
            completion_params(&notes, &dollars, 2),
        );
        hold.taken(client.sent.get());
        unanswered(
            client,
            &[late],
            "the stale catalog no longer holds the text",
        );
        hold.open();
        let late = result(answer(client, late, &mut Vec::new()));
        client.settle(&notes);
        (early, late)
    });
    let at = completion_params(&notes, &dollars, 2);
    let open = [(&*notes, &*dollars), (&*spec, LOGIN)];
    let fresh = fresh(&[&root], &open, "textDocument/completion", &[at]);
    for offered in [&early, &late] {
        assert!(
            !labels(offered).iter().any(|label| label == "FS-002-signin"),
            "the name between was offered: {offered}"
        );
    }
    assert_eq!(labels(&early), AFTER_FS_00);
    assert_eq!(late, fresh[0], "a fresh server answers `$$` otherwise");
}

/// Case 8, §FS-lsp.responsiveness.4 (R4): an edit to a file two folders' projects read holds the
/// completion in both, the one that walks it and the one whose stub names it.
#[test]
fn an_edit_to_a_file_two_projects_cover_holds_typing_in_both() {
    let root = test_root("typing_held_two_projects");
    let (a, b, target) = cross_folder_fixture(&root);
    let (a_note, b_use) = (a.join("docs/note.md"), b.join("docs/use.md"));
    let a_typed = "# Note\n\nSee $$FS-\n";
    let b_typed = "Uses \u{a7}FS-t here.\nSee $$FS-\n";
    write(&a_note, a_typed);
    let renamed = declaring("Disk lead.").replace("FS-t: T", "FS-u: U");
    let at = [
        completion_params(&a_note, a_typed, 2),
        completion_params(&b_use, b_typed, 1),
    ];
    let (mut server, connection) = server_over_folders(&[&a, &b]);
    let hold = server.hold_rebuilds();
    let (paths, edited, params) = (
        (a_note.clone(), b_use.clone(), target.clone()),
        renamed.clone(),
        at.clone(),
    );
    let (answers, _) = drive(server, connection, move |client| {
        let (a_note, b_use, target) = paths;
        let disk = declaring("Disk lead.");
        let open = [(&*target, &*disk), (&*a_note, a_typed), (&*b_use, b_typed)];
        opened_then_held(client, &hold, &open);
        client.change(&target, &edited, 2);
        let ids: Vec<i32> = params
            .into_iter()
            .map(|params| send(client, "textDocument/completion", params))
            .collect();
        hold.taken(client.sent.get());
        unanswered(client, &ids, "both projects read the renamed declaration");
        hold.open();
        ids.into_iter()
            .map(|id| result(answer(client, id, &mut Vec::new())))
            .collect::<Vec<_>>()
    });
    let open = [
        (&*target, &*renamed),
        (&*a_note, a_typed),
        (&*b_use, b_typed),
    ];
    let fresh = fresh(&[&a, &b], &open, "textDocument/completion", &at);
    assert_eq!(answers[0], fresh[0], "a's note");
    assert_eq!(answers[1], fresh[1], "b's use of the stub");
}
