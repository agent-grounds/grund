//! Repairs preserve authored row facts and offer only a reader that can read the file
//! (§FS-scan.checks.unread-scan-root.message, §FS-scan.checks.unread-scan-root.folder).

use super::unread_roots::unread_root_warnings;
use crate::config::load_config;
use crate::testing::{scan_tree, test_root, write};

#[test]
fn data_repairs_do_not_overwrite_forms_fetches_or_places() {
    for (name, keys, value, place) in [
        ("default", "", true, false),
        ("prose", "form = \"prose\"", false, false),
        ("fetch", "fetch = \"./fetcher\"", false, false),
        ("place", "citable = false", false, true),
    ] {
        let root = test_root(&format!("unread_data_repair_{name}"));
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n\
             [schema.kinds.FS]\nfiles = [\"fs.json\"]\n{keys}\n"
            ),
        );
        write(&root.join("fs.json"), "{\"a\": 1}\n");
        let config = load_config(&root).expect("project");
        let (findings, _) = scan_tree(&config, None, false).expect("scan");
        let warnings = unread_root_warnings(&config, &findings);
        assert_eq!(warnings.len(), 1, "{name}");
        let message = &warnings[0].message;
        assert!(message.contains("data language"), "{message}");
        assert_eq!(
            message.contains("Set `form = \"value\"`"),
            value,
            "{message}"
        );
        assert_eq!(message.contains("set `scan = false`"), place, "{message}");
        assert!(message.contains("listed document language"), "{message}");
    }
}

/// §FS-scan.checks.unread-scan-root.message: a hidden name a listed data language reads keeps
/// its own rename, and is never offered `form = "value"`.
#[test]
fn hidden_data_keeps_its_own_repair() {
    let root = test_root("unread_special_data_repairs");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n\
         [schema.kinds.FS]\nfiles = [\".fs.json\"]\n",
    );
    write(&root.join(".fs.json"), "body\n");
    let config = load_config(&root).expect("project");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let warnings = unread_root_warnings(&config, &findings);
    assert_eq!(warnings.len(), 1);
    let message = &warnings[0].message;
    assert!(
        message.ends_with(
            "Rename the file to a name that is not hidden and an extension a listed document \
             language reads."
        ),
        "{message}"
    );
    assert!(!message.contains("form = \"value\""), "{message}");
}

/// §FS-check.2.2.2: dropping a mapping is offered only when it restores an admitted text
/// reader, and byte order decides between several such mappings.
#[test]
fn empty_scan_caution_names_the_mapping_that_restores_a_text_reader() {
    for (name, sources, expected) in [
        (
            "order",
            "languages = [\"rust\", \"python\", \"json\"]\n\
            [schema.sources.extensions]\nrs = \"json\"\npy = \"json\"",
            "[schema.sources.extensions] maps `py` to `json`: drop that entry.",
        ),
        (
            "python",
            "languages = [\"python\", \"json\"]\n\
            [schema.sources.extensions]\npy = \"json\"",
            "[schema.sources.extensions] maps `py` to `json`: drop that entry.",
        ),
        (
            "unadmitted",
            "languages = [\"json\"]\n\
            [schema.sources.extensions]\npy = \"json\"",
            "admit one in [schema.sources] languages, such as `markdown`.",
        ),
    ] {
        let root = test_root(&format!("empty_scan_data_mapping_{name}"));
        write(
            &root.join("grund.toml"),
            &format!("grund_config_version = 2\n[schema.sources]\n{sources}\n"),
        );
        let config = load_config(&root).expect("project");
        let diagnostic =
            super::scope_cautions::empty_scan_warning(&config, &root, false, config.display());
        assert!(
            diagnostic.message.ends_with(expected),
            "{}",
            diagnostic.message
        );
        assert!(
            diagnostic
                .message
                .contains("no language in [schema.sources] languages is read for declarations")
        );
    }
}

/// §FS-scan.checks.unread-scan-root.folder: an admission repair must make this folder read;
/// a data language alone cannot read a prose home's declarations.
#[test]
fn an_unclaimed_data_suffix_is_not_offered_a_data_only_admission() {
    let root = test_root("unread_folder_unadmitted_data_repair");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.sources]\n\
        languages = [\"markdown\"]\n[schema.kinds.FS]\nfolders = [\"fs\"]\nindex = false\n",
    );
    write(&root.join("fs/a.json"), "{}\n");
    let config = load_config(&root).expect("project");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let warnings = unread_root_warnings(&config, &findings);
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings[0]
            .message
            .ends_with("Give the files in it an extension a listed language reads.")
    );
    assert!(!warnings[0].message.contains("Add `json`"));
    std::fs::rename(root.join("fs/a.json"), root.join("fs/a.md")).expect("apply the rename repair");
    let (repaired, _) = scan_tree(&config, None, false).expect("scan the repair");
    assert!(unread_root_warnings(&config, &repaired).is_empty());
}

fn warning(root: &std::path::Path) -> String {
    let config = load_config(root).expect("valid unrepaired configuration");
    let (findings, errors) = scan_tree(&config, None, false).expect("scan before repair");
    assert!(errors.is_empty(), "{errors:?}");
    let warnings = unread_root_warnings(&config, &findings);
    assert_eq!(warnings.len(), 1);
    warnings[0].message.clone()
}

fn assert_repaired(root: &std::path::Path) {
    let config = load_config(root).expect("the offered edit leaves the configuration valid");
    let (findings, errors) = scan_tree(&config, None, false).expect("scan the complete edit");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(unread_root_warnings(&config, &findings).is_empty());
    assert!(
        findings
            .declarations
            .keys()
            .any(|id| { id.kind == "FS" && id.slug.as_deref() == Some("a") }),
        "the intended FS-a declaration resolves: {:?}",
        findings.declarations
    );
}

/// §FS-scan.checks.unread-scan-root.message: fields and every authored closed strength
/// survive the offered Markdown-home repair; the eligible control can actually take value form.
#[test]
fn unread_value_form_repairs_preserve_fields_closed_and_existing_prerequisites() {
    for (name, keys, eligible) in [
        (
            "field",
            "[schema.kinds.FS.fields.terms]\npresence = \"must\"",
            false,
        ),
        ("closed", "closed = \"must\"", false),
        ("closed_may", "closed = \"may\"", false),
        ("eligible", "", true),
    ] {
        let root = test_root(&format!("unread_repair_validity_{name}"));
        let toml = format!(
            "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n[schema.kinds.FS]\nfiles = [\"fs.json\"]\n{keys}\n"
        );
        write(&root.join("grund.toml"), &toml);
        write(
            &root.join("fs.json"),
            if eligible {
                "{\"FS-a\": [1]}\n"
            } else {
                "# FS-a: A\n\n## terms: Terms\n"
            },
        );
        let message = warning(&root);
        assert_eq!(
            message.contains("Set `form = \"value\"`"),
            eligible,
            "{message}"
        );
        assert!(
            message.contains("give the file an extension a listed document language reads")
                || message.contains("Give the file an extension a listed document language reads"),
            "{message}"
        );
        if eligible {
            write(
                &root.join("grund.toml"),
                &format!("{toml}form = \"value\"\n"),
            );
        } else {
            std::fs::rename(root.join("fs.json"), root.join("fs.md")).expect("rename repair");
            write(&root.join("grund.toml"), &toml.replace("fs.json", "fs.md"));
        }
        assert_repaired(&root);
    }
}

/// §FS-scan.checks.unread-scan-root.message: a rename with no text reader includes admission,
/// and a hidden name still needs the rename.
#[test]
fn unread_rename_repairs_admit_a_reader_and_unhide_the_name() {
    for (file, languages) in [
        ("fs.json", "[\"json\"]"),
        (".fs.json", "[\"json\"]"),
        ("fs.dat", "[\"json\"]"),
        (".fs.md", "[\"json\"]"),
        (".fs.json", "[\"json\", \"markdown\"]"),
    ] {
        let root = test_root(&format!(
            "unread_rename_admission_{file}_{}",
            languages.len()
        ));
        let toml = format!(
            "grund_config_version = 2\n[schema.sources]\nlanguages = {languages}\n[schema.kinds.FS]\nfiles = [\"{file}\"]\nform = \"prose\"\n"
        );
        write(&root.join("grund.toml"), &toml);
        write(&root.join(file), "# FS-a: A\n");
        let message = warning(&root);
        assert_eq!(
            message.contains("add `markdown`") || message.contains("Add `markdown`"),
            languages == "[\"json\"]",
            "{message}"
        );
        if file.starts_with('.') {
            assert!(
                message.contains("ename the file to a name that is not hidden"),
                "{message}"
            );
        }
        std::fs::rename(root.join(file), root.join("fs.md")).expect("offered rename");
        let repaired = toml
            .replace("[\"json\"]", "[\"json\", \"markdown\"]")
            .replace(&format!("[\"{file}\"]"), "[\"fs.md\"]");
        write(&root.join("grund.toml"), &repaired);
        assert_repaired(&root);
    }
}

/// §FS-scan.checks.unread-scan-root.folder: the offered repair for a Markdown rule restores a
/// document reading, never a source admission. A rule folder no admitted language reads as
/// Markdown is refused at load (§FS-config-v2.checks.config-rule-folder-not-markdown), so a
/// document reader is always admitted here.
#[test]
fn unread_rule_folder_source_suffix_repairs_read_the_markdown_rule() {
    let root = test_root("unread_rule_source_suffix_document_admitted");
    let toml = "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\"]\n\
                [schema.kinds.FS]\n[schema.kinds.RULE]\nfolders = [\"rules\"]\n\
                index = false\nform = \"rule\"\n";
    write(&root.join("grund.toml"), toml);
    write(
        &root.join("rules/terms.rs"),
        "# RULE-terms: Each FS must cite at least one RULE.\n\nEvery spec is held to a rule.\n",
    );
    let message = warning(&root);
    assert!(
        message.ends_with("Give the files in it an extension a listed document language reads."),
        "{message}"
    );
    assert!(!message.contains("Add `rust`"), "{message}");
    assert!(
        !message.contains(", or "),
        "only one repair offered: {message}"
    );

    // Apply the sole offered edit.
    std::fs::rename(root.join("rules/terms.rs"), root.join("rules/terms.md"))
        .expect("offered document rename");
    let config = load_config(&root).expect("complete repair preserves loader prerequisites");
    assert!(super::config_warnings(&config).is_empty());
    assert!(config.reading(&root.join("rules/terms.md")).markdown);
    let (findings, errors) = scan_tree(&config, None, false).expect("repaired rule scan");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(unread_root_warnings(&config, &findings).is_empty());
    assert!(
        findings
            .declarations
            .keys()
            .any(|id| { id.kind == "RULE" && id.slug.as_deref() == Some("terms") }),
        "RULE-terms resolves: {:?}",
        findings.declarations
    );
}

/// §FS-scan.checks.unread-scan-root.folder: source admission remains useful for a non-rule
/// folder holding a declaration in the source reader's doc comments.
#[test]
fn unread_non_rule_folder_keeps_the_working_builtin_source_admission() {
    let root = test_root("unread_prose_source_admission");
    let toml = "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\"]\n\
                [schema.kinds.FS]\nfolders = [\"fs\"]\nindex = false\nform = \"prose\"\n";
    write(&root.join("grund.toml"), toml);
    write(&root.join("fs/a.rs"), "/// FS-a: A\nfn a() {}\n");
    assert!(warning(&root).ends_with(
        "Add `rust` to [schema.sources] languages, or give the files in it an extension a listed language reads."
    ));
    write(
        &root.join("grund.toml"),
        &toml.replace("[\"markdown\"]", "[\"markdown\", \"rust\"]"),
    );
    assert_repaired(&root);
}
