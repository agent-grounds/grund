//! Test module: the empty-folder obligation warning (§FS-check.2.2.1) — one
//! warning per empty obligation kind for a citable folder whose scanned content
//! goes past its entry files.

use super::*;
use crate::config::load_config;
use crate::testing::{check_run, scan_tree, test_root, write};

/// §FS-check.2.2.1: a citable folder with scanned content beyond each of its
/// effective entry files earns one warning per empty obligation kind. This
/// covers the default README index, a named index, and `index = false`, plus
/// the `should` wording and the once-per-kind boundary.
#[test]
fn empty_folder_obligations_warn_once_with_each_entry_rule() {
    let root = test_root("empty_folder_obligations_warn_once_with_each_entry_rule");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 1

[[kinds]]
kind = "FS"
folder = "docs/spec"
index = false

[[kinds]]
kind = "DEFAULT"
folder = "docs/default"

[[kinds]]
kind = "CUSTOM"
folder = "docs/custom"
index = "INDEX.md"

[[kinds]]
kind = "NOIDX"
folder = "docs/no-index"
index = false

[citations.DEFAULT]
must = ["FS"]
[citations.CUSTOM]
should = ["FS"]
[citations.NOIDX]
must = ["FS"]
"#,
    );
    write(&root.join("docs/default/README.md"), "# Default index\n");
    write(&root.join("docs/default/one.md"), "# First\n");
    write(&root.join("docs/default/two.md"), "# Second\n");
    write(&root.join("docs/custom/INDEX.md"), "# Custom index\n");
    write(&root.join("docs/custom/one.md"), "# First\n");
    write(&root.join("docs/no-index/README.md"), "# Content\n");

    let config = load_config(&root).expect("load config");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan");
    let report = check_findings(&findings, &config);
    let warnings = report
        .warnings
        .iter()
        .filter(|warning| warning.code == "empty-citation-obligation")
        .collect::<Vec<_>>();

    assert_eq!(warnings.len(), 3, "one warning per qualifying kind");
    assert!(warnings.iter().all(|warning| {
        warning.path.is_none() && warning.line.is_none() && warning.sites.is_empty()
    }));
    assert!(warnings.iter().any(|warning| {
            warning.message
                == "[citations.DEFAULT] must applies to nothing — docs/default/ declares no DEFAULT ID; did you mean `citable = false`?"
        }));
    assert!(warnings.iter().any(|warning| {
            warning.message
                == "[citations.CUSTOM] should applies to nothing — docs/custom/ declares no CUSTOM ID; did you mean `citable = false`?"
        }));
    assert!(warnings.iter().any(|warning| {
            warning.message
                == "[citations.NOIDX] must applies to nothing — docs/no-index/ declares no NOIDX ID; did you mean `citable = false`?"
        }));
    assert!(
        report.suggestions.is_empty(),
        "zero units have no should miss"
    );
}

/// §FS-check.2.2.1.1: entry-only folders, file homes, prohibition-only tables,
/// the homeless kind, and unwalked kinds stay silent even when a full walk
/// makes the unwalked home visible.
#[test]
fn empty_folder_obligation_warning_negative_matrix() {
    let root = test_root("empty_folder_obligation_warning_negative_matrix");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 1

[[kinds]]
kind = "FS"
folder = "docs/spec"
index = false

[[kinds]]
kind = "ENTRY"
folder = "entry"

[[kinds]]
kind = "SINGLE"
file = "docs/single.md"

[[kinds]]
kind = "PROHIBIT"
folder = "prohibit"
index = false

[[kinds]]
kind = "src"
citable = false

[[kinds]]
kind = "UNWALK"
folder = "unwalk"
citable = false
scan = false

[citations.ENTRY]
must = ["FS"]
[citations.SINGLE]
must = ["FS"]
[citations.PROHIBIT]
must-not = ["FS"]
[citations.src]
must = ["FS"]
"#,
    );
    write(&root.join("entry/README.md"), "# Entry-only scaffold\n");
    write(&root.join("docs/single.md"), "# Single-file scaffold\n");
    write(
        &root.join("prohibit/extra.md"),
        "# Prohibition-only content\n",
    );
    write(&root.join("src/main.rs"), "fn main() {}\n");
    write(&root.join("unwalk/extra.md"), "# Unwalked content\n");

    let mut config = load_config(&root).expect("load config");
    // Config validation rejects an obligation on an unwalked kind because it
    // can never fire. Inject it after loading to pin the checker's defensive
    // exclusion independently of that load-time rule.
    let unwalk_rule = config
        .citations
        .per_kind
        .get("ENTRY")
        .cloned()
        .expect("ENTRY rule");
    config
        .citations
        .per_kind
        .insert("UNWALK".to_string(), unwalk_rule);
    config.scan_full = true;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("full scan");
    assert!(
        findings
            .scanned_files
            .iter()
            .any(|file| file.ends_with("unwalk/extra.md")),
        "the full walk must expose the unwalked negative case"
    );
    let report = check_findings(&findings, &config);

    assert!(
        !report
            .warnings
            .iter()
            .any(|warning| warning.code == "empty-citation-obligation")
    );
}

/// §FS-check.2.2.1: the run-level warning is independent of unrelated graph
/// errors, including a dangling citation in another scanned file.
#[test]
fn empty_folder_obligation_warning_coexists_with_other_findings() {
    let root = test_root("empty_folder_obligation_warning_coexists_with_other_findings");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 1

[[kinds]]
kind = "FS"
folder = "docs/spec"
index = false

[[kinds]]
kind = "SKILL"
folder = "skills"
index = false

[citations.SKILL]
must = ["FS"]
"#,
    );
    write(&root.join("skills/README.md"), "# Skills\n");
    write(&root.join("skills/review.md"), "# Review\n");
    write(&root.join("src/app.rs"), "// see §FS-001-missing\n");

    let config = load_config(&root).expect("load config");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan");
    let report = check_findings(&findings, &config);

    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "empty-citation-obligation")
    );
    assert!(report.errors.iter().any(|error| error.code == "dangling"));
}

/// §FS-check.2.2.1.3 / §FS-workspace.5: empty-obligation warnings are evaluated
/// per workspace member, so an entry-only member does not inherit a sibling's
/// warning.
#[test]
fn empty_folder_obligation_warning_is_isolated_per_workspace_member() {
    let root = test_root("empty_folder_obligation_warning_is_isolated_per_workspace_member");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 1
project_name = "root"

[[kinds]]
kind = "FS"
folder = "docs/spec"
index = false

[[kinds]]
kind = "SKILL"
folder = "skills"

[citations.SKILL]
must = ["FS"]

[workspace]
members = ["member"]
"#,
    );
    write(
        &root.join("member/grund.toml"),
        r#"grund_config_version = 1
project_name = "member"

[[kinds]]
kind = "FS"
folder = "docs/spec"
index = false

[[kinds]]
kind = "SKILL"
folder = "skills"

[citations.SKILL]
must = ["FS"]
"#,
    );
    write(&root.join("skills/README.md"), "# Skills\n");
    write(&root.join("skills/review.md"), "# Review\n");
    write(&root.join("member/skills/README.md"), "# Member skills\n");

    let run = check_run(&root, false);
    let warnings = run
        .report
        .warnings
        .iter()
        .filter(|warning| warning.code == "empty-citation-obligation")
        .collect::<Vec<_>>();

    assert_eq!(
        warnings.len(),
        1,
        "only the root member has non-entry content"
    );
    assert!(warnings[0].message.contains("skills/"));
}
