//! Writer regressions for rendered ordering, replacement metadata and the bytes outside the owned
//! declaration under
//! [§FS-fetch.4](../../docs/functional-spec/FS-fetch.md#4-file-home-write) and
//! [§FS-fetch.6](../../docs/functional-spec/FS-fetch.md#6-stability-and-ownership), and for a final
//! bare CR, the one line ending the validator and the write path read alike under
//! [§FS-fetch.3.3](../../docs/functional-spec/FS-fetch.md#33-a-heading-with-no-text-is-still-a-heading).
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use support::*;

#[test]
fn external_facts_file_home_orders_by_the_effective_rendered_id() {
    let root = root("file-rendered-order");
    write(
        &root,
        "grund.toml",
        concat!(
            "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"TICKET\"\nfile = \"docs/tickets.md\"\n",
            "format = \"{kind}-{slug}-{number}\"\nfetch = \"scripts/fetch-ticket\"\n",
            "[scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        ),
    );
    write(
        &root,
        "docs/tickets.md",
        "# Tickets\n\n## TICKET-zeta-1: Existing\n\nBody.\n",
    );
    executable(
        &root,
        "scripts/fetch-ticket",
        "#!/bin/sh\nprintf '## TICKET-alpha-9: Inserted\\n\\nBody.\\n'\n",
    );

    assert_code(&run(&root, &["fetch", "TICKET-alpha-9"]), 0, "insert");
    assert_eq!(
        fs::read_to_string(root.join("docs/tickets.md")).unwrap(),
        concat!(
            "# Tickets\n\n",
            "## TICKET-alpha-9: Inserted\n\nBody.\n\n",
            "## TICKET-zeta-1: Existing\n\nBody.\n"
        )
    );
}

#[test]
fn external_facts_replacement_preserves_file_and_folder_home_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let file = root("file-mode-preservation");
    write(
        &file,
        "grund.toml",
        &file_config("docs/tickets.md", None, Some("scripts/fetch-ticket")),
    );
    write(
        &file,
        "docs/tickets.md",
        "# Tickets\n\n## TICKET-8: Old\n\nOld.\n",
    );
    fs::set_permissions(
        file.join("docs/tickets.md"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    executable(
        &file,
        "scripts/fetch-ticket",
        "#!/bin/sh\nprintf '## TICKET-8: New\\n\\nNew.\\n'\n",
    );
    assert_code(&run(&file, &["fetch", "TICKET-8"]), 0, "file replace");
    assert_eq!(
        fs::metadata(file.join("docs/tickets.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let folder = root("folder-mode-preservation");
    write(
        &folder,
        "grund.toml",
        concat!(
            "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"TICKET\"\nfolder = \"snapshots\"\nindex = false\n",
            "format = \"{kind}-{number}\"\nfetch = \"scripts/fetch-ticket\"\n",
            "[scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        ),
    );
    write(
        &folder,
        "snapshots/provider.md",
        "# TICKET-8: Old\n\nOld.\n",
    );
    fs::set_permissions(
        folder.join("snapshots/provider.md"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    executable(
        &folder,
        "scripts/fetch-ticket",
        "#!/bin/sh\nprintf '# TICKET-8: New\\n\\nNew.\\n'\n",
    );
    assert_code(&run(&folder, &["fetch", "TICKET-8"]), 0, "folder replace");
    assert_eq!(
        fs::metadata(folder.join("snapshots/provider.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}

/// A repository of one `AR` kind whose fetcher prints `payload`'s bytes as they are.
fn arch_repo(name: &str, home: &str, payload: &str) -> std::path::PathBuf {
    let root = root(name);
    write(
        &root,
        "grund.toml",
        &format!(
            "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n\
             [[kinds]]\nkind = \"AR\"\n{home}\nfetch = \"scripts/fetch\"\n\n\
             [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        ),
    );
    write(&root, "scripts/payload", payload);
    executable(&root, "scripts/fetch", "#!/bin/sh\ncat scripts/payload\n");
    root
}

/// Fetch `AR-001-thing` over a `file` home twice, and hold the home to `after` both times.
fn assert_fetched_home(name: &str, before: &str, after: &str) {
    let root = arch_repo(
        name,
        "file = \"docs/arch.md\"",
        "## AR-001-thing: Thing\nnew\n",
    );
    write(&root, "docs/arch.md", before);
    for fetch in ["first fetch", "second fetch"] {
        assert_code(&run(&root, &["fetch", "AR-001-thing"]), 0, name);
        assert_eq!(
            fs::read_to_string(root.join("docs/arch.md")).unwrap(),
            after,
            "{name}: {fetch}"
        );
    }
}

/// §FS-fetch.4, §FS-declarations.body: a replacement stops where the next scan ends the owned
/// declaration, so what comes after it is not replaced with it, on either fetch. Each row is the
/// home's bytes after `old\n` before the fetch, and after `new\n` once it has run.
#[test]
fn external_facts_replacement_keeps_every_byte_after_the_owned_declaration() {
    const OTHER: &str = "\n\n## AR-002-other: Other\nother\n";
    for (index, (tail, kept)) in [
        (
            "\n# Part two\n\ntext of part two",
            "\n# Part two\n\ntext of part two",
        ),
        ("# Part two\ntext", "\n# Part two\ntext"),
        // Its indent is the line's own: this boundary starts where the line does.
        ("\n   # Part two\n\ntext", "\n   # Part two\n\ntext"),
        // Bare headings the scan ends the body at, at the declaration's depth and above it.
        ("#\ntext after", "\n#\ntext after"),
        ("##\ntext after", "\n##\ntext after"),
        // Prose with a colon: no malformed declaration in the home, and no ID to match.
        (
            "\n# Architecture: overview\n\ntext",
            "\n# Architecture: overview\n\ntext",
        ),
        // Sibling declarations above the depth, as written and however indented.
        (
            "\n# AR-003-third: Third\nthird",
            "\n# AR-003-third: Third\nthird",
        ),
        (
            "\n    # AR-003-third: Third\nthird",
            "\n    # AR-003-third: Third\nthird",
        ),
        (
            "\n\t# AR-003-third: Third\nthird",
            "\n\t# AR-003-third: Third\nthird",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert_fetched_home(
            &format!("file-boundary-{index}"),
            &format!("# Architecture\n\n## AR-001-thing: Thing\nold\n{tail}{OTHER}"),
            &format!("# Architecture\n\n## AR-001-thing: Thing\nnew\n{kept}{OTHER}"),
        );
    }
}

/// §FS-fetch.4: an H1 carrying the requested ID is not the H2 a `file` home replaces, so the
/// fetch inserts in ID order and leaves that line alone.
#[test]
fn external_facts_an_h1_of_the_requested_id_is_not_the_declaration_replaced() {
    assert_fetched_home(
        "file-h1-requested-id",
        "# AR-001-thing: Thing\nold\n\n## AR-002-other: Other\nother\n",
        "# AR-001-thing: Thing\nold\n\n## AR-001-thing: Thing\nnew\n\n\
         ## AR-002-other: Other\nother\n",
    );
}

/// §FS-fetch.4: an insertion goes in ID order past a heading above the depth, which it leaves.
#[test]
fn external_facts_an_insertion_keeps_a_heading_above_the_depth() {
    assert_fetched_home(
        "file-insert-past-h1",
        "## AR-000-first: First\nfirst\n\n# Part two\n\ntext\n\n## AR-002-other: Other\nother\n",
        "## AR-000-first: First\nfirst\n\n# Part two\n\ntext\n\n\
         ## AR-001-thing: Thing\nnew\n\n## AR-002-other: Other\nother\n",
    );
}

const FILE_HOME: &str = "file = \"docs/arch.md\"";
const FOLDER_HOME: &str = "folder = \"docs/arch\"";

/// §FS-fetch.3.3: a final CR is a line ending to the validator as it is to the write path, so a
/// bare heading in the output's last bytes is refused with the heading named without it. Exit 2,
/// and nothing written.
#[test]
fn external_facts_a_bare_heading_ending_in_a_final_cr_is_refused_with_the_tree_unchanged() {
    let bare = |heading: &str| {
        format!(
            "error: fetch output line 3: heading `{heading}` has no text and cannot stay in \
             `AR-001-thing`; fence it or drop it to keep the text after it\n"
        )
    };
    for (index, (home, declaration, last, refusal)) in [
        (FILE_HOME, "##", "##\r", bare("##")),
        (FILE_HOME, "##", "#\r", bare("#")),
        (FILE_HOME, "##", "####\r", bare("####")),
        (FILE_HOME, "##", "   ##\r", bare("   ##")),
        (FOLDER_HOME, "#", "#\r", bare("#")),
        (
            FILE_HOME,
            "##",
            "###\r",
            // §FS-fetch.3.2 keeps a bare heading at subsection depth.
            "error: fetch output line 3: heading `###` is not a section of `AR-001-thing`; \
             number it, fence it, or make it a bold label\n"
                .to_string(),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let root = arch_repo(
            &format!("final-cr-refused-{index}"),
            home,
            &format!("{declaration} AR-001-thing: Thing\nkept\n{last}"),
        );
        let before = tree(&root);
        let fetched = run(&root, &["fetch", "AR-001-thing"]);
        assert_code(&fetched, 2, last);
        assert_eq!(stderr(&fetched), refusal, "{last:?}");
        assert_eq!(tree(&root), before, "{last:?} changed the tree");
    }
}

/// The final CR moves only what the scan reads as a heading: these stay accepted, written as they
/// are. Their idempotence is another matter, so this holds the first fetch alone.
#[test]
fn external_facts_a_line_ending_in_a_final_cr_the_scan_reads_as_content_stays_accepted() {
    for (index, last) in ["    ##\r", "kept\r"].into_iter().enumerate() {
        let payload = format!("## AR-001-thing: Thing\nkept\n{last}");
        let root = arch_repo(&format!("final-cr-accepted-{index}"), FILE_HOME, &payload);
        assert_code(&run(&root, &["fetch", "AR-001-thing"]), 0, last);
        assert_eq!(
            fs::read_to_string(root.join("docs/arch.md")).unwrap(),
            payload,
            "{last:?}"
        );
    }
}
