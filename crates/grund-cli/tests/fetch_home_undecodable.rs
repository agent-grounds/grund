//! A home file that cannot be decoded refuses the fetch with the line `grund check` prints for
//! it, in a folder home (§FS-fetch.5.4) and a file home (§FS-fetch.4.4). What the fetch prints
//! is compared to what `check` prints on the same tree, which a case's goldens cannot say.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::path::Path;
use support::*;

/// `# FS-a: Alpha`, then a byte no UTF-8 sequence starts with.
const BROKEN: &[u8] = b"# FS-a: Alpha\n\xff";

fn folder_project(test: &str) -> std::path::PathBuf {
    let project = root(&format!("home-undecodable-{test}"));
    write(
        &project,
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n\
         [schema.kinds.FS]\nfolders = [\"fs\"]\nindex = false\nfetch = \"scripts/fetch-const\"\n",
    );
    executable(
        &project,
        "scripts/fetch-const",
        "#!/bin/sh\nprintf '# FS-a: Alpha\\n\\nFetched body.\\n'\n",
    );
    write(&project, "fs/a.md", "# FS-a: Alpha\n\nLocal body.\n");
    write(&project, "fs/b.md", "# FS-b: Beta\n\nBody.\n");
    write(&project, "fs/c.md", "# FS-c: Gamma\n\nBody.\n");
    project
}

fn file_project(test: &str) -> std::path::PathBuf {
    let project = root(&format!("home-undecodable-{test}"));
    write(
        &project,
        "grund.toml",
        &file_config("docs/tickets.md", None, Some("scripts/fetch-const")),
    );
    executable(
        &project,
        "scripts/fetch-const",
        "#!/bin/sh\nprintf '## TICKET-1: One\\n\\nFetched body.\\n'\n",
    );
    write(
        &project,
        "docs/tickets.md",
        "# Tickets\n\n## TICKET-1: One\n\nLocal body.\n",
    );
    project
}

fn corrupt(project: &Path, relative: &str, bytes: &[u8]) {
    fs::write(project.join(relative), bytes).expect("write the undecodable file");
}

/// §FS-fetch.5.4: run `check`, then `fetch` from `cwd`; the fetch must refuse with the very line
/// `check` printed for `shown`, nothing on stdout and the tree untouched.
fn assert_refused_with_checks_line(project: &Path, cwd: &Path, id: &str, shown: &str) {
    let check = run(cwd, &["check"]);
    assert_code(&check, 2, "check on the broken tree");
    let checked = stderr(&check);
    let prefix = format!("error: {shown}: ");
    let line = checked
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("check names {shown}: {checked}"))
        .to_owned();
    let before = tree_with_directories(project);
    let fetch = run(cwd, &["fetch", id]);
    assert_code(&fetch, 2, "fetch");
    assert_eq!(stdout(&fetch), "", "fetch stdout");
    assert_eq!(stderr(&fetch).trim_end(), line, "the line check prints");
    assert_eq!(
        tree_with_directories(project),
        before,
        "refusal changed the tree"
    );
}

#[test]
#[ignore = "red until #1309 names the undecodable file; implement removes this"]
fn the_requested_ids_own_file_is_named_with_the_line_check_prints() {
    let project = folder_project("own");
    corrupt(&project, "fs/a.md", BROKEN);
    assert_refused_with_checks_line(&project, &project, "FS-a", "fs/a.md");
}

#[test]
#[ignore = "red until #1309 names the undecodable file; implement removes this"]
fn a_sibling_of_the_requested_ids_file_is_named() {
    let project = folder_project("sibling");
    corrupt(&project, "fs/b.md", b"# FS-b: Beta\n\xff");
    assert_refused_with_checks_line(&project, &project, "FS-a", "fs/b.md");
}

/// §FS-fetch.5.4: `check` prints both lines; the fetch stops at the first in the walk's order.
#[test]
#[ignore = "red until #1309 names the undecodable file; implement removes this"]
fn the_first_of_several_undecodable_files_is_named() {
    let project = folder_project("several");
    corrupt(&project, "fs/b.md", b"# FS-b: Beta\n\xff");
    corrupt(&project, "fs/c.md", b"# FS-c: Gamma\n\xff");
    assert_refused_with_checks_line(&project, &project, "FS-a", "fs/b.md");
}

/// §FS-cli.3.4: the default base is the config root wherever the run stands, and
/// `relative_paths = false` is the invocation base (§FS-config.3.6.1).
#[test]
#[ignore = "red until #1309 names the undecodable file; implement removes this"]
fn the_file_is_spelled_from_the_report_base() {
    let project = folder_project("base");
    corrupt(&project, "fs/a.md", BROKEN);
    assert_refused_with_checks_line(&project, &project.join("fs"), "FS-a", "fs/a.md");

    // `fetch` takes no `--path-base`, so the version 1 key is the one way to move that base.
    let config = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/e2e/cases/fetch-folder-source-v1-python-refused/repo/grund.toml"),
    )
    .expect("read the version 1 config");
    let project = root("home-undecodable-base-v1");
    write(
        &project,
        "grund.toml",
        &format!("{config}\n[output]\nrelative_paths = false\n"),
    );
    executable(
        &project,
        "scripts/fetch",
        "#!/bin/sh\nprintf '# FS-001-a: Alpha\\n\\nFetched body.\\n'\n",
    );
    fs::create_dir_all(project.join("fs")).expect("create the home");
    corrupt(&project, "fs/a.md", b"# FS-001-a: Alpha\n\xff");
    assert_refused_with_checks_line(&project, &project.join("fs"), "FS-001-a", "a.md");
}

/// §FS-fetch.4.4: the same refusal through the file home's own read of the bytes.
#[test]
#[ignore = "red until #1309 names the undecodable file; implement removes this"]
fn an_undecodable_file_home_is_named() {
    let project = file_project("file-home");
    corrupt(
        &project,
        "docs/tickets.md",
        b"# Tickets\n\n## TICKET-1: One\n\xff",
    );
    assert_refused_with_checks_line(&project, &project, "TICKET-1", "docs/tickets.md");
}

/// The controls: the same projects fetch once every file decodes, so the refusals above are
/// about the byte. They pass today.
#[test]
fn a_decodable_home_fetches() {
    let folder = folder_project("control-folder");
    assert_code(&run(&folder, &["fetch", "FS-a"]), 0, "folder home");
    let file = file_project("control-file");
    assert_code(&run(&file, &["fetch", "TICKET-1"]), 0, "file home");
}
