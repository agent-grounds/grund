//! Folder-home value-source refusal and its successful controls (§FS-fetch.5.1).
//! Port issue #1139's intake reproducer through the real CLI and shared tree helpers.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use support::*;

const ID: &str = "CONST-room-limit";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"vals\"\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\
    fetch = \"scripts/fetch-const\"\n[rules.resolution]\nCONST = \"warn\"\n";
const TABLE: &str =
    "{\n  \"CONST-room-limit\": [40, \"rooms\"],\n  \"CONST-floor-limit\": [7, \"floors\"]\n}\n";
const SNAPSHOT: &str = "# CONST-room-limit: Fetched\n\n## 1. 40\n\n## 2. rooms\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1139-values-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create isolated fixture");
        let fixture = Self(root);
        write(&fixture.0, "grund.toml", CONFIG);
        write(&fixture.0, "unrelated.bin", "unchanged\0\r\n");
        write(&fixture.0, "values/keep.txt", "other home content\r\n");
        executable(
            &fixture.0,
            "scripts/fetch-const",
            "#!/bin/sh\nprintf '# %s: Fetched\\n\\n## 1. 40\\n\\n## 2. rooms\\n' \"$1\"\n",
        );
        fixture
    }

    fn source(&self, path: &str, body: &str, index: &str) {
        write(&self.0, path, body);
        write(&self.0, "values/README.md", index);
    }

    fn json(&self) {
        self.source(
            "values/limits.json",
            TABLE,
            "# Values\n\n- [§CONST-room-limit](limits.json)\n- [§CONST-floor-limit](limits.json)\n",
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_usable(root: &Path) {
    let check = run(root, &["check"]);
    assert_code(&check, 0, "check without duplicate");
    assert!(!stdout(&check).contains("duplicate declaration"));
    let show = run(root, &["show", ID]);
    assert_code(&show, 0, "unambiguous show");
    assert!(!stderr(&show).contains("ambiguous ID"));
}

fn assert_refused(root: &Path, source: &str, clean: bool) {
    if clean {
        assert_usable(root);
    } else {
        let check = run(root, &["check"]);
        assert_code(&check, 1, "collision exists before fetch");
        assert!(stdout(&check).contains("duplicate declaration"));
        assert!(stdout(&check).contains(source));
    }
    let before = tree_with_directories(root);
    let show_before = run(root, &["show", ID]);
    if !clean {
        assert_code(&show_before, 1, "collision is already ambiguous");
        assert!(stderr(&show_before).contains("ambiguous ID"));
    }
    let output = run(root, &["fetch", ID]);
    // Run both readers even on the broken implementation, so red evidence names
    // the duplicate this regression pins rather than just the changed exit.
    let check = run(root, &["check"]);
    let show = run(root, &["show", ID]);
    assert_code(
        &output,
        2,
        &format!(
            "value-source refusal; post-fetch check={:?}: {}; show={:?}: {}",
            check.status.code(),
            stdout(&check),
            show.status.code(),
            stderr(&show)
        ),
    );
    assert!(
        output.stdout.is_empty(),
        "fetch stdout: {}",
        stdout(&output)
    );
    let message = stderr(&output);
    assert!(message.contains(ID), "missing requested ID: {message}");
    assert!(
        message.contains(source),
        "missing declaring source: {message}"
    );
    assert!(
        message.contains("move") || message.contains("keep"),
        "missing remedy: {message}"
    );
    assert_eq!(tree_with_directories(root), before, "refusal changed tree");
    assert_eq!(
        show.stdout, show_before.stdout,
        "show changed after refusal"
    );
    if clean {
        assert_usable(root);
    } else {
        // An existing Markdown/value collision stays ambiguous; fetch must not
        // mutate either declaration in an attempt to repair that authored tree.
        assert_code(&check, 1, "preexisting duplicate remains");
        assert_code(&show, 1, "preexisting ambiguity remains");
    }
}

/// §FS-fetch.5.1: default scan input omits JSON, but the catalog reads its members.
#[test]
fn folder_value_source_default_json_refuses_without_changing_tree() {
    let fixture = Fixture::new();
    fixture.json();
    assert_refused(&fixture.0, "values/limits.json", true);
}

/// §FS-values.2.2, §FS-fetch.5.1: scan exclusions and ignore files cannot hide a value.
#[test]
fn folder_value_source_json_refusal_is_independent_of_scan_filters() {
    let fixture = Fixture::new();
    fixture.json();
    write(
        &fixture.0,
        "grund.toml",
        &format!(
            "{CONFIG}\n[schema.sources]\nlanguages = [\"markdown\"]\n\
             exclude = [\"values/limits.json\"]\nrespect_gitignore = true\n"
        ),
    );
    write(&fixture.0, ".gitignore", "values/limits.json\n");
    assert_refused(&fixture.0, "values/limits.json", true);
}

/// §FS-fetch.5.1: finding Markdown must not short-circuit value-source discovery.
#[test]
fn folder_value_source_markdown_collision_still_refuses_unchanged() {
    let fixture = Fixture::new();
    fixture.json();
    write(
        &fixture.0,
        "values/room-limit-notes.md",
        "# CONST-room-limit: Authored\n\n## 1. 39\n\n## 2. rooms\n",
    );
    assert_refused(&fixture.0, "values/limits.json", false);
}

fn mapped_data_refuses(extension: &str) {
    let fixture = Fixture::new();
    write(
        &fixture.0,
        "grund.toml",
        &format!(
            "{CONFIG}\n[schema.sources]\nlanguages = [\"markdown\", \"table\"]\n\
             [schema.sources.extensions]\n{extension} = \"table\"\n\
             [schema.sources.definitions.table]\nclass = \"data\"\n"
        ),
    );
    let source = format!("values/limits.{extension}");
    fixture.source(
        &source,
        TABLE,
        &format!(
            "# Values\n\n- [§CONST-room-limit](limits.{extension})\n\
             - [§CONST-floor-limit](limits.{extension})\n"
        ),
    );
    assert_refused(&fixture.0, &source, true);
}

/// §FS-config-v2.schema.sources.values, §FS-fetch.5.1: data language, not suffix.
#[test]
fn folder_value_source_mapped_data_refuses() {
    mapped_data_refuses("table");
}

/// §FS-fetch.5.1: mapped data is JSON even with a YAML suffix; already recognized.
#[test]
fn folder_value_source_mapped_yaml_data_refuses() {
    mapped_data_refuses("yaml");
}

/// §FS-fetch.5.1: the same existing language rule applies to the short YAML suffix.
#[test]
fn folder_value_source_mapped_yml_data_refuses() {
    mapped_data_refuses("yml");
}

/// §FS-fetch.5: the reproducer's differently named Markdown replacement control.
#[test]
fn folder_value_source_control_markdown_is_replaced() {
    let fixture = Fixture::new();
    fixture.source(
        "values/room-limit-notes.md",
        "# CONST-room-limit: Old\n\n## 1. 39\n\n## 2. rooms\n",
        "# Values\n\n- [§CONST-room-limit](room-limit-notes.md)\n",
    );
    let mut expected = tree_with_directories(&fixture.0);
    expected.insert(
        "values/room-limit-notes.md".into(),
        Some(SNAPSHOT.as_bytes().to_vec()),
    );
    let output = run(&fixture.0, &["fetch", ID]);
    assert_code(&output, 0, "replace differently named Markdown");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(tree_with_directories(&fixture.0), expected);
    assert_usable(&fixture.0);
}

/// §FS-fetch.5.1: other IDs, nested/outside tables and unrelated findings do not gate fetch.
#[test]
fn folder_value_source_control_absent_id_is_created() {
    let fixture = Fixture::new();
    fixture.source(
        "values/limits.json",
        "{\"CONST-floor-limit\": [7, \"floors\"], \"CONST-desk-limit\": [2]}\n",
        "# Values\n\n- [§CONST-floor-limit](limits.json)\n\
         - [§CONST-desk-limit](limits.json)\n- [§CONST-room-limit](CONST-room-limit.md)\n",
    );
    // Neither is a direct child of this kind's home (§FS-values.2.2).
    write(&fixture.0, "values/nested/limits.json", TABLE);
    write(&fixture.0, "outside/limits.json", TABLE);
    assert_code(&run(&fixture.0, &["show", ID]), 1, "ID is absent");
    let mut expected = tree_with_directories(&fixture.0);
    expected.insert(
        "values/CONST-room-limit.md".into(),
        Some(SNAPSHOT.as_bytes().to_vec()),
    );
    let output = run(&fixture.0, &["fetch", ID]);
    assert_code(&output, 0, "create genuinely absent ID");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(tree_with_directories(&fixture.0), expected);
    assert_usable(&fixture.0);

    write(
        &fixture.0,
        "values/unrelated.json",
        "{\"CONST-unrelated\": []}\n",
    );
    assert_code(&run(&fixture.0, &["check"]), 1, "unrelated finding");
    let before = tree_with_directories(&fixture.0);
    assert_code(
        &run(&fixture.0, &["fetch", ID]),
        0,
        "unrelated finding does not gate fetch",
    );
    assert_eq!(tree_with_directories(&fixture.0), before);
}

/// §FS-fetch.1, §FS-fetch.5.1: a different project's same ID does not block the owner.
#[test]
fn folder_value_source_control_other_project_does_not_block_creation() {
    let fixture = Fixture::new();
    fixture.json();
    write(
        &fixture.0,
        "grund.toml",
        &format!("{CONFIG}\n[workspace]\nmembers = [\"member\"]\n"),
    );
    write(
        &fixture.0,
        "member/grund.toml",
        &CONFIG.replace("project_name = \"vals\"", "project_name = \"member\""),
    );
    write(
        &fixture.0,
        "member/values/README.md",
        "# Values\n\n- [§CONST-room-limit](CONST-room-limit.md)\n",
    );
    executable(
        &fixture.0,
        "member/scripts/fetch-const",
        "#!/bin/sh\nprintf '# %s: Fetched\\n\\n## 1. 40\\n\\n## 2. rooms\\n' \"$1\"\n",
    );
    let mut expected = tree_with_directories(&fixture.0);
    expected.insert(
        "member/values/CONST-room-limit.md".into(),
        Some(SNAPSHOT.as_bytes().to_vec()),
    );
    let output = run(&fixture.0, &["fetch", "member/CONST-room-limit"]);
    assert_code(&output, 0, "select member, not root's value source");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(tree_with_directories(&fixture.0), expected);
    assert_usable(&fixture.0.join("member"));
    assert_usable(&fixture.0);
}

/// §FS-fetch.5.1, §FS-values.2.2.4: a fetched kind's YAML source declares its IDs, so it refuses.
#[test]
fn folder_value_source_yaml_and_yml_refuse_without_changing_tree() {
    for extension in ["yaml", "yml"] {
        let fixture = Fixture::new();
        let source = format!("values/limits.{extension}");
        fixture.source(
            &source,
            "CONST-room-limit: [40, rooms]\nCONST-floor-limit: [7, floors]\n",
            &format!(
                "# Values\n\n- [§CONST-room-limit](limits.{extension})\n\
                 - [§CONST-floor-limit](limits.{extension})\n"
            ),
        );
        assert_refused(&fixture.0, &source, true);
    }
}

/// §FS-fetch.5.1: a YAML source declaring only other IDs does not prevent creation.
#[test]
fn folder_value_source_control_yaml_and_yml_with_other_ids_allow_creation() {
    for extension in ["yaml", "yml"] {
        let fixture = Fixture::new();
        fixture.source(
            &format!("values/limits.{extension}"),
            "CONST-floor-limit: [7, floors]\n",
            &format!(
                "# Values\n\n- [§CONST-floor-limit](limits.{extension})\n\
                 - [§CONST-room-limit](CONST-room-limit.md)\n"
            ),
        );
        assert_code(&run(&fixture.0, &["check"]), 0, "no YAML finding");
        assert_code(&run(&fixture.0, &["show", ID]), 1, "ID is absent");
        let mut expected = tree_with_directories(&fixture.0);
        expected.insert(
            "values/CONST-room-limit.md".into(),
            Some(SNAPSHOT.as_bytes().to_vec()),
        );
        let output = run(&fixture.0, &["fetch", ID]);
        assert_code(&output, 0, "other IDs permit creation");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
        assert_eq!(tree_with_directories(&fixture.0), expected);
        assert_usable(&fixture.0);
    }
}

/// §FS-fetch.5, §FS-values.2.2.4: an unparsable source stops fetch as it stops every reader.
#[test]
fn folder_value_source_unparsable_yaml_and_yml_refuse_without_changing_tree() {
    for extension in ["yaml", "yml"] {
        let fixture = Fixture::new();
        let source = format!("values/limits.{extension}");
        fixture.source(
            &source,
            "CONST-room-limit: [40, rooms\n",
            &format!("# Values\n\n- [§CONST-room-limit](limits.{extension})\n"),
        );
        assert_code(&run(&fixture.0, &["check"]), 2, "check stops on the source");
        assert_code(
            &run(&fixture.0, &["show", ID]),
            2,
            "show stops on the source",
        );
        let before = tree_with_directories(&fixture.0);
        let output = run(&fixture.0, &["fetch", ID]);
        assert_code(&output, 2, "fetch stops on the source");
        assert!(
            output.stdout.is_empty(),
            "fetch stdout: {}",
            stdout(&output)
        );
        let message = stderr(&output);
        assert!(message.contains(&source), "missing source: {message}");
        assert!(
            message.contains("invalid YAML"),
            "missing reason: {message}"
        );
        assert_eq!(
            tree_with_directories(&fixture.0),
            before,
            "refusal changed tree"
        );
    }
}

fn add_alias_home(fixture: &Fixture, index_target: &str) {
    write(
        &fixture.0,
        "grund.toml",
        &format!("{CONFIG}\n[schema.kinds.OTHER]\nfolders = [\"a\"]\nform = \"value\"\n"),
    );
    write(
        &fixture.0,
        "a/README.md",
        &format!(
            "# Values\n\n- [§CONST-room-limit]({index_target})\n\
             - [§CONST-floor-limit]({index_target})\n"
        ),
    );
}

/// §FS-fetch.5.1: fetch and the catalog agree on an aliased source. Its first spelling picks
/// the reader, here YAML, which takes this JSON text and so declares the IDs for both.
#[test]
fn folder_value_source_yaml_alias_refuses_as_the_catalog_recognizes_it() {
    let fixture = Fixture::new();
    fixture.json();
    add_alias_home(&fixture, "limits.yaml");
    let alias = fixture.0.join("a/limits.yaml");
    let target = Path::new("../values/limits.json");
    std::os::unix::fs::symlink(target, &alias).expect("link YAML alias");

    let check = run(&fixture.0, &["check"]);
    assert_code(&check, 1, "catalog recognizes the multiply owned source");
    assert!(
        stdout(&check).contains("a/limits.yaml:2: error: CONST-room-limit declares kind CONST")
    );
    assert!(stdout(&check).contains("duplicate declaration of CONST-room-limit"));
    let show = run(&fixture.0, &["show", ID]);
    assert_code(&show, 1, "recognized source is already ambiguous");
    assert!(stderr(&show).contains("a/limits.yaml:2"));

    assert_refused(&fixture.0, "a/limits.yaml", false);
    assert_eq!(run(&fixture.0, &["check"]).stdout, check.stdout);
    assert_eq!(run(&fixture.0, &["show", ID]).stderr, show.stderr);
    assert!(!fixture.0.join("values/CONST-room-limit.md").exists());
    assert_eq!(fs::read_link(&alias).unwrap(), target);
}

/// §FS-fetch.5.1: the catalog's first JSON alias enrolls with all owners before refusal.
#[test]
fn folder_value_source_json_alias_refuses_creation_and_replacement() {
    let fixture = Fixture::new();
    fixture.source(
        "a/limits.json",
        TABLE,
        "# Values\n\n- [§CONST-room-limit](limits.yaml)\n- [§CONST-floor-limit](limits.yaml)\n",
    );
    add_alias_home(&fixture, "limits.json");
    let alias = fixture.0.join("values/limits.yaml");
    let target = Path::new("../a/limits.json");
    std::os::unix::fs::symlink(target, &alias).expect("link selected YAML spelling");

    let check = run(&fixture.0, &["check"]);
    assert_code(
        &check,
        1,
        "catalog recognizes the multiply owned JSON source",
    );
    assert!(stdout(&check).contains("declares kind CONST inside OTHER home a"));
    assert!(stdout(&check).contains("duplicate declaration of CONST-room-limit"));
    let show = run(&fixture.0, &["show", ID]);
    assert_code(&show, 1, "recognized source is already ambiguous");
    assert!(stderr(&show).contains("a/limits.json:2"));
    assert_refused(&fixture.0, "a/limits.json", false);
    assert_eq!(run(&fixture.0, &["check"]).stdout, check.stdout);
    assert_eq!(run(&fixture.0, &["show", ID]).stderr, show.stderr);
    assert!(!fixture.0.join("values/CONST-room-limit.md").exists());

    // The same recognized group must also prevent replacement of authored Markdown.
    write(
        &fixture.0,
        "values/room-limit-notes.md",
        "# CONST-room-limit: Authored\n\n## 1. 39\n\n## 2. rooms\n",
    );
    assert_refused(&fixture.0, "a/limits.json", false);
    assert_eq!(fs::read_link(&alias).unwrap(), target);
}
