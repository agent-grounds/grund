//! A buffer is the file at its physical location even where that file exists and cannot be stated
//! (§FS-scan.identity.buffers): a value home that is a directory link to a folder listed but not
//! searched enrolls an edited buffer once, whichever spelling the editor opened it under.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::LspSnapshotOpts;
use crate::api::lsp_snapshot;
use crate::testing::{DirectoryMode, test_root, write};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"v\"\n\
    [schema.kinds.CONST]\nfolders = [\"alias\"]\nform = \"value\"\nindex = false\n";

fn value(id: &str) -> String {
    format!("{{\"{id}\": [40, \"rooms\"]}}\n")
}

/// What the editor's run declares with `a.json` edited and `new.json` written only in a buffer,
/// both opened under `spelling`, and how many scan errors it met.
fn declared(root: &Path, spelling: &str) -> (Vec<String>, usize) {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.to_path_buf(),
        path_provided: true,
        open_documents: BTreeMap::from([
            (root.join(spelling).join("a.json"), value("CONST-edited")),
            (root.join(spelling).join("new.json"), value("CONST-new")),
        ]),
    })
    .expect("the editor's snapshot");
    let ids = snapshot
        .declarations
        .into_iter()
        .map(|d| d.query_id)
        .collect();
    (ids, snapshot.scan_errors.len())
}

#[test]
fn an_edited_buffer_in_a_home_that_cannot_be_searched_is_enrolled_once() {
    let root = test_root("overlay_unstatable_home");
    write(&root.join("grund.toml"), CONFIG);
    write(&root.join("values/a.json"), &value("CONST-saved"));
    std::os::unix::fs::symlink("values", root.join("alias")).expect("link the home");
    let expected = (vec!["CONST-edited".to_string(), "CONST-new".to_string()], 0);

    for spelling in ["values", "alias"] {
        assert_eq!(declared(&root, spelling), expected, "readable, {spelling}");
    }

    let _barred = DirectoryMode::set(&root.join("values"), 0o444);
    let denied = fs::metadata(root.join("values/a.json"))
        .is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied);
    if !denied {
        eprintln!("skipped: stating the file was not denied, as when running as root");
        return;
    }
    for spelling in ["values", "alias"] {
        assert_eq!(declared(&root, spelling), expected, "denied, {spelling}");
    }
}
