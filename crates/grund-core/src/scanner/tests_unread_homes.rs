//! Folder evidence and the shared fetch refusal follow the scan's reading, in either version
//! (§FS-scan.checks.unread-scan-root, §FS-fetch.4.2.1, §FS-fetch.5.5).

use super::fetch_refused_home;
use crate::config::{Config, RefusedHome, load_config};
use crate::model::{ExcludingRule, RootOrigin};
use crate::testing::{TemporaryRoot, scan_tree, test_root, write};

fn project(name: &str, sources: &str, row: &str) -> (TemporaryRoot, Config) {
    let root = test_root(name);
    let index = if row.contains("folders =") && !row.contains("citable = false") {
        "index = false\n"
    } else {
        ""
    };
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\n[schema.sources]\n{sources}\n[schema.kinds.FS]\n{row}\n{index}"
        ),
    );
    let config = load_config(&root).expect("the project loads");
    (root, config)
}

#[test]
fn folder_evidence_uses_first_eligible_file_and_every_home() {
    let (root, config) = project(
        "folder_evidence_order",
        "languages = [\"markdown\"]\nexclude = [\"**/excluded/**\"]",
        "folders = [\"fs\", \"other\"]",
    );
    for file in [
        "fs/z.dat",
        "fs/a.txt",
        "fs/.hidden",
        "fs/excluded/first.dat",
        "other/a.dat",
    ] {
        write(&root.join(file), "unread\n");
    }
    write(&root.join("fs/.ignore"), "ignored\n");
    write(&root.join("fs/ignored/first.dat"), "unread\n");
    let (findings, errors) = scan_tree(&config, None, false).expect("scan");
    assert!(errors.is_empty());
    let rows = &findings.unread_roots;
    assert_eq!(rows.len(), 2);
    for (home, file) in [("fs", "fs/a.txt"), ("other", "other/a.dat")] {
        let row = rows
            .iter()
            .find(|row| row.written == home)
            .expect("each home");
        // The loaded root may be spelled differently from the fixture's (a Windows 8.3
        // temp name), so compare the components below it.
        assert!(
            matches!(&row.origin, RootOrigin::FolderHome { first_refused, .. }
            if first_refused.ends_with(file)),
            "{home}: {:?}",
            row.origin
        );
        assert_eq!(row.config, std::path::Path::new("grund.toml"));
        assert_eq!(row.line, 6);
    }
}

#[test]
fn folder_evidence_stays_silent_without_eligible_refusals_or_after_a_read() {
    for (name, row, files) in [
        ("absent", "folders = [\"fs\"]", vec![]),
        ("empty", "folders = [\"fs\"]", vec![]),
        ("hidden", "folders = [\"fs\"]", vec![("fs/.gitkeep", "")]),
        (
            "excluded",
            "folders = [\"fs\"]",
            vec![("fs/excluded/a.dat", "")],
        ),
        (
            "ignored",
            "folders = [\"fs\"]",
            vec![("fs/.ignore", "a.dat\n"), ("fs/a.dat", "")],
        ),
        (
            "unwalked",
            "folders = [\"fs\"]\ncitable = false\nscan = false",
            vec![("fs/a.dat", "")],
        ),
        (
            "text",
            "folders = [\"fs\"]",
            vec![("fs/a.dat", ""), ("fs/a.md", "# FS-a: A\n")],
        ),
        (
            "value",
            "folders = [\"fs\"]\nform = \"value\"",
            vec![("fs/a.dat", ""), ("fs/a.json", "{\"a\": 1}")],
        ),
    ] {
        let root = test_root(&format!("folder_evidence_{name}"));
        if name == "empty" {
            std::fs::create_dir_all(root.join("fs")).expect("empty home");
        }
        for (file, text) in files {
            write(&root.join(file), text);
        }
        let index = if row.contains("citable = false") {
            ""
        } else {
            "index = false\n"
        };
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"json\"]\nexclude = [\"fs/excluded\"]\n[schema.kinds.FS]\n{row}\n{index}"
            ),
        );
        let config = load_config(&root).expect("project");
        let (findings, _) = scan_tree(&config, None, false).expect("scan");
        assert!(
            findings.unread_roots.is_empty(),
            "{name}: {:?}",
            findings.unread_roots
        );
    }
}

#[test]
fn version_one_folder_evidence_and_fetch_share_the_extension_reason() {
    let root = test_root("v1_unread_folder_and_fetch");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\
        [scan]\ninclude = [\"src\"]\nextensions = [\"rs\"]\n\
        [[kinds]]\nkind = \"FS\"\nfolder = \"fs\"\nindex = false\nfetch = \"./fetcher\"\n",
    );
    write(&root.join("fs/a.md"), "# FS-001-a: A\n");
    let config = load_config(&root).expect("project");
    let (findings, errors) = scan_tree(&config, None, false).expect("scan");
    assert!(errors.is_empty());
    assert_eq!(findings.unread_roots.len(), 1);
    assert_eq!(findings.unread_roots[0].written, "fs");
    assert_eq!(findings.unread_roots[0].line, 7);
    let (_, reason) = fetch_refused_home(config.schema(), config.frame(), "FS", Some("FS-002-b"))
        .expect("Markdown is not in the extension list");
    assert!(matches!(reason, RefusedHome::Unread(ref why)
        if why == "extension md is not in [scan] extensions"));
}

#[test]
fn fetch_refusal_uses_each_reading_and_hidden_composition() {
    for (file, class) in [
        ("fs.md", "markdown"),
        ("fs.py", "source"),
        ("fs.json", "data"),
        ("fs.dat", "unclaimed"),
        (".fs.json", "hidden-data"),
    ] {
        let (_root, config) = project(
            &format!("fetch_reading_{class}"),
            "languages = [\"markdown\", \"python\", \"json\"]",
            &format!("files = [\"{file}\"]\nfetch = \"./fetcher\""),
        );
        let refused = fetch_refused_home(config.schema(), config.frame(), "FS", None);
        match (class, refused) {
            ("markdown", None) | ("source", Some((_, RefusedHome::SourceFile))) => {}
            ("data", Some((_, RefusedHome::Unread(reason)))) => {
                assert!(reason.contains("data language"))
            }
            ("unclaimed", Some((_, RefusedHome::Unread(reason)))) => {
                assert!(reason.contains("no language"))
            }
            ("hidden-data", Some((_, RefusedHome::Unread(reason)))) => {
                assert!(reason.contains("its name begins with `.`"));
                assert!(reason.contains("data language"));
            }
            _ => panic!("wrong refusal for {class}"),
        }
    }
}

#[test]
fn fetch_folder_target_asks_exclude_ignore_and_unclaimed_only() {
    for (name, sources, ignored) in [
        (
            "exclude",
            "languages = [\"markdown\"]\nexclude = [\"fs/FS-a.md\"]",
            false,
        ),
        ("ignore", "languages = [\"markdown\"]", true),
    ] {
        let (root, config) = project(
            &format!("fetch_target_{name}"),
            sources,
            "folders = [\"fs\"]\nfetch = \"./fetcher\"",
        );
        write(&root.join("fs/.gitkeep"), "");
        if ignored {
            write(&root.join(".ignore"), "fs/FS-a.md\n");
        }
        assert!(fetch_refused_home(config.schema(), config.frame(), "FS", None).is_none());
        let (_, refused) = fetch_refused_home(config.schema(), config.frame(), "FS", Some("FS-a"))
            .expect("the unclaimed target is excluded");
        assert!(matches!(
            (ignored, refused),
            (false, RefusedHome::Excluded(ExcludingRule::Pattern(_)))
                | (
                    true,
                    RefusedHome::Excluded(ExcludingRule::IgnoreFile { .. })
                )
        ));
    }
}
