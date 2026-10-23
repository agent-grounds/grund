//! Test module: a citation on a declaration heading in a kind's own index is a
//! citation from the declaration that heading declares, never the entry
//! (§FS-check.4.1.2.1, §FS-citations.recognized.title-lines). The entry rule
//! and the unused carve-out give it the same answer (agent-grounds/grund#956).

use std::path::PathBuf;

use super::tests_kind_index_enrollment::scanned_index;
use crate::api::{ListOpts, list};
use crate::testing::{check_run, findings, test_root, write};

/// The issue's tree. The index's own heading cites `FS-session`, which the index
/// does not list. The control is `FS-login`'s heading, which cites `FS-token`,
/// unlisted too: the same title citation, outside the index.
fn title_citation_repo(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"fixture\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.kinds.FS]\nfolders = [\"docs/fs\"]\n",
    );
    write(
        &root.join("docs/fs/FS-login.md"),
        "# FS-login: User login, after [\u{a7}FS-token](FS-token.md#fs-token-user-token)\n",
    );
    write(
        &root.join("docs/fs/FS-session.md"),
        "# FS-session: User session\n",
    );
    write(
        &root.join("docs/fs/FS-token.md"),
        "# FS-token: User token\n",
    );
    write(
        &root.join("docs/fs/README.md"),
        "# FS-index: Index, after [\u{a7}FS-session](FS-session.md#fs-session-user-session)\n\n\
         - [\u{a7}FS-index](README.md#fs-index-index-after-fs-session)\n\
         - [\u{a7}FS-login](FS-login.md#fs-login-user-login-after-fs-token)\n",
    );
    root
}

/// §FS-check.4.1.2.1: the title citation on the index's line 1 is not the entry,
/// while the list entry on line 4 still is.
#[test]
fn a_title_citation_in_its_own_index_is_not_the_entry() {
    let root = title_citation_repo("a_title_citation_in_its_own_index_is_not_the_entry");
    let (_, findings, entries) = scanned_index(&root);
    let in_index = |line: usize| {
        findings
            .citations
            .iter()
            .find(|citation| citation.file.ends_with("docs/fs/README.md") && citation.line == line)
            .unwrap_or_else(|| panic!("a citation at README.md:{line}"))
    };
    assert!(
        entries.is_index_entry(in_index(4)),
        "the list entry is the entry"
    );
    assert!(
        !entries.is_index_entry(in_index(1)),
        "a citation on a declaration heading is never the entry"
    );
}

/// §FS-check.4.1.2.1 / §FS-check.3.18.5.1: one run gives one answer. The index
/// holds no entry for `FS-session`, and its title citation is use, exactly as
/// `FS-login`'s title citation of `FS-token` is.
#[test]
fn a_title_citation_in_its_own_index_counts_as_use() {
    let root = title_citation_repo("a_title_citation_in_its_own_index_counts_as_use");
    let run = check_run(&root, false);
    let unlisted: Vec<&str> = run
        .report
        .errors
        .iter()
        .filter(|diagnostic| diagnostic.code == "missing-index-entry")
        .filter_map(|diagnostic| diagnostic.message.split(' ').next())
        .collect();
    assert_eq!(
        unlisted,
        ["FS-session", "FS-token"],
        "neither is listed: {:?}",
        findings(&run)
    );
    let unused: Vec<&str> = run
        .report
        .warnings
        .iter()
        .filter(|diagnostic| diagnostic.code == "unused")
        .filter_map(|diagnostic| {
            diagnostic
                .message
                .strip_prefix("declared but never cited: ")
        })
        .filter_map(|tail| tail.split(' ').next())
        .collect();
    assert_eq!(
        unused,
        ["FS-login", "FS-index"],
        "both title citations are use: {:?}",
        findings(&run)
    );
}

/// §FS-list.1.3: `list --unused` discounts the entry within the same bounds as
/// `check`, so the title citation keeps `FS-session` off the list.
#[test]
fn list_unused_counts_a_title_citation_in_its_own_index() {
    let root = title_citation_repo("list_unused_counts_a_title_citation_in_its_own_index");
    let output = list(ListOpts {
        path: root.clone(),
        path_provided: true,
        unused_only: true,
        ..ListOpts::default()
    })
    .expect("list the fixture");
    let ids: Vec<&str> = output
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["FS-index", "FS-login"],
        "list --unused agrees with check"
    );
}
