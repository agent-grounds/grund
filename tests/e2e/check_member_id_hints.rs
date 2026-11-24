//! §FS-check.3.1.5: #1489's root/member reproducer and diagnostic boundaries.
#[path = "check_member_id_hints/support.rs"]
mod support;
use support::*;

#[test]
fn check_member_id_hints_original_text() {
    let tree = Tree::new();
    let run = tree.run("", &["check"]);
    assert_eq!(run.status.code(), Some(1));
    assert!(run.stderr.is_empty());
    assert!(stdout(&run).lines().any(|line| line == format!(
        "{ROOT}:4: error: {}", hinted("AR-001-roo", "AR-001-root"))), "{run:?}");
    let expected = format!("{ROOT}:3: error: {}", hinted(ID, &format!("worldsim/{ID}")));
    let actual = stdout(&run)
        .lines()
        .find(|line| line.starts_with(&format!("{ROOT}:3:")))
        .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn check_member_id_hints_original_json() {
    let tree = Tree::new();
    let rows = tree.check("", None);
    dangling(at(&rows, ROOT, 4), &hinted("AR-001-roo", "AR-001-root"));
    let row = at(&rows, ROOT, 3);
    assert_eq!(
        row.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "authority",
            "code",
            "line",
            "message",
            "path",
            "severity",
            "sites"
        ]
    );
    assert!(row["sites"].is_null() && row["authority"].is_null());
    dangling(row, &hinted(ID, &format!("worldsim/{ID}")));
}

#[test]
fn check_member_id_hints_show_and_local_control() {
    let tree = Tree::new();
    let run = tree.run("", &[ID]);
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(stdout(&run), "");
    assert_eq!(
        stderr(&run),
        format!("ID not found: {ID}; did you mean worldsim/{ID}?\n")
    );
    let run = tree.run("", &[&format!("worldsim/{ID}")]);
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(stdout(&run), "The monitor.\n");
    assert_eq!(stderr(&run), "");
    dangling(
        at(&tree.check("", None), ROOT, 4),
        &hinted("AR-001-roo", "AR-001-root"),
    );
}

#[test]
fn check_member_id_hints_alternatives_grammar_and_section() {
    let tree = Tree::new();
    for (alias, numbered) in [
        ("zeta", false),
        ("beta", true),
        ("alpha", false),
        ("delta", false),
    ] {
        tree.project(alias, numbered);
        tree.declare(alias, ID);
    }
    tree.members(&["zeta", "beta", "alpha", "delta", "worldsim"]);
    tree.citation(&format!("{MARK}{ID}.9"));
    // The same text is slug 005-invariant-monitor in root/alpha, but number
    // 005 plus slug invariant-monitor in beta. No project has section 9.
    let alternatives = format!("alpha/{ID}, beta/{ID} or delta/{ID}");
    let show = tree.run("", &[&format!("{ID}.9")]);
    assert_eq!(show.status.code(), Some(1));
    assert_eq!(
        stderr(&show),
        format!("ID not found: {ID}; did you mean {alternatives}?\n")
    );
    dangling(
        at(&tree.check("", None), ROOT, 3),
        &hinted(ID, &alternatives),
    );
}

#[test]
fn check_member_id_hints_member_separator_owns_written_coordinate() {
    let tree = Tree::new();
    let id = "AR-monitor.9";
    let candidate = format!("worldsim/{id}");
    tree.edit(
        "worldsim/grund.toml",
        "slug_pattern = \"[a-z0-9][a-z0-9-]*\"",
        "section_separator = \"_\"\nslug_pattern = \"[a-z0-9][a-z0-9.-]*\"",
    );
    tree.declare("worldsim", id);
    tree.citation(&format!("{MARK}{id}"));
    // Root reads .9 as a section; the member reads it as part of the ID.
    let show = tree.run("", &[id]);
    assert_eq!(show.status.code(), Some(1));
    assert_eq!(stdout(&show), "");
    assert_eq!(
        stderr(&show),
        format!("ID not found: AR-monitor; did you mean {candidate}?\n")
    );
    let qualified = tree.run("", &[&candidate]);
    assert_eq!(qualified.status.code(), Some(0));
    assert_eq!(stdout(&qualified), "The monitor.\n");
    assert_eq!(stderr(&qualified), "");
    let message = hinted("AR-monitor", &candidate);
    dangling(at(&tree.check("", None), ROOT, 3), &message);

    // The full-tree tier must pass the same complete coordinate to the rule.
    tree.edit(
        "grund.toml",
        "\n[workspace]",
        "\n[schema.kinds.code]\nfolders = [\"src\"]\n\n[workspace]",
    );
    tree.write("notes/citation.md", &format!("{MARK}{id}\n"));
    let full = tree.run("", &["check", "--full", "--format", "json"]);
    assert_eq!(full.status.code(), Some(1), "{full:?}");
    assert_eq!(stderr(&full), "");
    let rows: Vec<serde_json::Value> = stdout(&full)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    dangling(at(&rows, ROOT, 3), &message);
    let outside = at(&rows, "notes/citation.md", 1);
    assert_eq!(outside["code"], "out-of-scope-dangling", "{outside}");
    assert_eq!(outside["severity"], "error", "{outside}");
    assert_eq!(
        outside["message"],
        format!("outside configured places and source scope: {message}")
    );
}

#[test]
fn check_member_id_hints_member_precedes_local() {
    let tree = Tree::new();
    tree.declare("", "AR-005-invariant-monitors");
    tree.citation(&format!("{MARK}{ID}"));
    dangling(
        at(&tree.check("", None), ROOT, 3),
        &hinted(ID, &format!("worldsim/{ID}")),
    );
}

#[test]
fn check_member_id_hints_inline_configured_marker() {
    let tree = Tree::new();
    tree.edit("grund.toml", "[schema]\n", "[schema]\nmarker = \"@\"\n");
    tree.edit("docs/ar/README.md", MARK, "@");
    tree.declare("", "AR-005-invariant-monitors");
    tree.citation(&format!("`@{ID}`"));
    dangling(
        at(&tree.check("", None), ROOT, 3),
        &format!(
            "{} (or write <@>{ID} if this is an illustration)",
            hinted(ID, &format!("worldsim/{ID}"))
        ),
    );
}

#[test]
fn check_member_id_hints_missing_places() {
    let tree = Tree::new();
    tree.edit(
        "grund.toml",
        "folders = [\"docs/ar\"]",
        "folders = [\"absent\"]",
    );
    // Citations still read from the source complement, even though the home is absent.
    tree.write("code.md", &format!("{MARK}{ID}\n"));
    dangling(
        at(&tree.check("", None), "code.md", 1),
        &hinted(ID, &format!("worldsim/{ID}")),
    );
}

fn fetch_tail(inline: bool) {
    let tree = Tree::new();
    // v1 remains the supported way to configure fetch. Read-only check must
    // never execute the deliberately nonexistent integration.
    tree.write("grund.toml", "grund_config_version = 1\n\n[id]\nslug_pattern = \"[a-z0-9][a-z0-9-]*\"\n\n[[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\nfetch = \"grund-1489-must-not-run\"\nresolve = \"must\"\n\n[scan]\ninclude = [\"docs/ar\"]\n\n[workspace]\nmembers = [\"worldsim\"]\n");
    tree.citation(&if inline {
        format!("`{MARK}{ID}`")
    } else {
        format!("{MARK}{ID}")
    });
    let escape = if inline {
        format!(" (or write <{MARK}>{ID} if this is an illustration)")
    } else {
        String::new()
    };
    dangling(
        at(&tree.check("", None), ROOT, 3),
        &format!(
            "unknown reference {ID}; no snapshot in docs/ar; did you mean worldsim/{ID}?{escape}"
        ),
    );
}

#[test]
fn check_member_id_hints_fetch_plain_tail() {
    fetch_tail(false);
}

#[test]
fn check_member_id_hints_fetch_inline_tail() {
    fetch_tail(true);
}

#[test]
fn check_member_id_hints_root_path_report_scope() {
    let tree = Tree::new();
    tree.write(
        "docs/ar/AR-other.md",
        &format!("# AR-other: Other\n\n{MARK}AR-absent\n"),
    );
    // Resolve all root declarations even though only ROOT's findings are reported.
    tree.write("docs/ar/AR-outside.md", "# AR-outside: Outside\n");
    tree.citation(&format!("{MARK}{ID}\n{MARK}AR-outside"));
    let rows = tree.check("", Some(ROOT));
    assert!(rows.iter().all(|row| row["path"] == ROOT), "{rows:?}");
    assert!(!rows.iter().any(|row| row["line"] == 4), "{rows:?}");
    dangling(at(&rows, ROOT, 3), &hinted(ID, &format!("worldsim/{ID}")));
}

#[test]
fn check_member_id_hints_root_directory_report_scope() {
    let tree = Tree::new();
    tree.write(
        "notes/citations.md",
        &format!("{MARK}{ID}\n{MARK}AR-001-root\n"),
    );
    let rows = tree.check("", Some("notes"));
    assert!(
        rows.iter().all(|row| row["path"] == "notes/citations.md"),
        "{rows:?}"
    );
    assert!(!rows.iter().any(|row| row["line"] == 2), "{rows:?}");
    dangling(
        at(&rows, "notes/citations.md", 1),
        &hinted(ID, &format!("worldsim/{ID}")),
    );
}

#[test]
fn check_member_id_hints_member_citing_sibling() {
    let tree = Tree::new();
    tree.project("consumer", false);
    tree.members(&["worldsim", "consumer"]);
    tree.write(
        "consumer/docs/ar/AR-consumer.md",
        &format!("# AR-consumer: Consumer\n\n{MARK}{ID}\n{MARK}AR-consumer\n"),
    );
    dangling(
        at(&tree.check("", None), "consumer/docs/ar/AR-consumer.md", 3),
        &hinted(ID, &format!("worldsim/{ID}")),
    );
    // This member's own declaration is resolved locally and never suggested
    // back to itself just because root does not declare it.
    assert!(
        !tree
            .check("", None)
            .iter()
            .any(|row| row["path"] == "consumer/docs/ar/AR-consumer.md" && row["line"] == 4)
    );
}

#[test]
fn check_member_id_hints_member_citing_root() {
    let tree = Tree::new();
    tree.edit(
        "grund.toml",
        "grund_config_version = 2",
        "grund_config_version = 2\nproject_name = \"root\"",
    );
    tree.declare("", ID);
    tree.project("consumer", false);
    tree.members(&["worldsim", "consumer"]);
    tree.write(
        "consumer/docs/ar/AR-consumer.md",
        &format!("# AR-consumer: Consumer\n\n{MARK}{ID}\n"),
    );
    dangling(
        at(&tree.check("", None), "consumer/docs/ar/AR-consumer.md", 3),
        &hinted(ID, &format!("root/{ID} or worldsim/{ID}")),
    );
}

#[test]
fn check_member_id_hints_unseen_projects_control() {
    let tree = Tree::new();
    tree.project("consumer", false);
    tree.members(&["worldsim", "consumer"]);
    tree.write(
        "consumer/docs/ar/AR-consumer.md",
        &format!("# AR-consumer: Consumer\n\n{MARK}{ID}\n"),
    );
    for (dir, path) in [
        ("consumer", None),
        ("", Some("consumer")),
        ("", Some("consumer/docs/ar")),
    ] {
        let rows = tree.check(dir, path);
        dangling(
            at(&rows, "docs/ar/AR-consumer.md", 3),
            &format!("unknown reference {ID}"),
        );
    }
    tree.edit(
        "grund.toml",
        "\n[workspace]\nmembers = [\"worldsim\", \"consumer\"]\n",
        "\n",
    );
    // Without a workspace boundary these directories would be scanned as
    // part of root's source complement, making the ID locally resolvable.
    std::fs::remove_dir_all(tree.0.join("worldsim")).unwrap();
    std::fs::remove_dir_all(tree.0.join("consumer")).unwrap();
    dangling(
        at(&tree.check("", None), ROOT, 3),
        &format!("unknown reference {ID}"),
    );
}

#[test]
fn check_member_id_hints_no_member_match_and_qualified_controls() {
    let tree = Tree::new();
    tree.declare("", "AR-005-invariant-monitors");
    tree.citation(&format!("`{MARK}AR-005-invariant-monito`\n{MARK}worldsim/AR-005-invariant-monito\n{MARK}AR-001-root.9"));
    let rows = tree.check("", None);
    dangling(
        at(&rows, ROOT, 3),
        &format!(
            "{} (or write <{MARK}>AR-005-invariant-monito if this is an illustration)",
            hinted("AR-005-invariant-monito", "AR-005-invariant-monitors")
        ),
    );
    dangling(
        at(&rows, ROOT, 4),
        &hinted(
            "worldsim/AR-005-invariant-monito",
            &format!("worldsim/{ID}"),
        ),
    );
    let section = at(&rows, ROOT, 5);
    assert_eq!(section["code"], "missing-section", "{section}");
    assert!(!section["message"].as_str().unwrap().contains("worldsim/"));
}

#[test]
fn check_member_id_hints_root_kind_absent_control() {
    let tree = Tree::new();
    tree.edit("grund.toml", "[schema.kinds.AR]", "[schema.kinds.DA]");
    tree.write(ROOT, &format!("# DA-root: Root\n\n{MARK}{ID}\n"));
    tree.write(
        "docs/ar/README.md",
        &format!("# DA\n\n- [{MARK}DA-root](AR-001-root.md#da-root-root)\n"),
    );
    let run = tree.run("", &["check", "--format", "json"]);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert!(run.stderr.is_empty());
    let rows: Vec<serde_json::Value> = stdout(&run)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    // AR is unknown to the citing grammar, so its token is not recognized
    // at all, despite the loaded member's real AR declaration.
    assert!(
        !rows
            .iter()
            .any(|row| row["path"] == ROOT && row["line"] == 3),
        "{rows:?}"
    );
}
