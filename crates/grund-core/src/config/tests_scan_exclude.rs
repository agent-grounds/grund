//! Test module: the `[scan] exclude` component-name grammar — an entry
//! containing `/` is refused at the key's line (§FS-config.3.5.16).

use super::load_config;
use crate::testing::{test_root, write};

/// Load a config whose `[scan] exclude` is `list`, written on line 4.
fn load_exclude(name: &str, list: &str) -> anyhow::Result<Vec<String>> {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!("grund_config_version = 1\n\n[scan]\nexclude = {list}\n"),
    );
    load_config(&root).map(|config| config.exclude)
}

/// The located error for `entry`, offering `name` as the at-any-depth repair.
fn refusal(entry: &str, name: &str) -> String {
    format!(
        "grund.toml:4: [scan] exclude entry `{entry}` contains `/`, not a directory name; \
         remove it to keep the scan unchanged, or use `{name}` to exclude that name at any depth"
    )
}

fn load_err(name: &str, list: &str) -> String {
    match load_exclude(name, list) {
        Ok(exclude) => panic!("a `/` entry must fail to load, loaded {exclude:?}"),
        Err(err) => format!("{err:#}"),
    }
}

/// §FS-config.3.5.16: the slash entry is refused wherever it sits in the list,
/// located at the `exclude` key's line and naming the entry and both repairs.
#[test]
fn scan_exclude_refuses_a_slash_entry_first_middle_or_last() {
    for (case, list) in [
        ("first", r#"["docs/plans", "target", "build"]"#),
        ("middle", r#"["target", "docs/plans", "build"]"#),
        ("last", r#"["target", "build", "docs/plans"]"#),
    ] {
        let err = load_err(&format!("scan_exclude_refuses_slash_{case}"), list);
        assert_eq!(err, refusal("docs/plans", "plans"), "{case}");
    }
}

/// §FS-config.3.5.16: entries are judged in list order and only the first `/`
/// entry is reported; its suggested name is its last non-empty component.
#[test]
fn scan_exclude_reports_only_the_first_of_two_slash_entries() {
    let err = load_err(
        "scan_exclude_reports_only_the_first_of_two_slash_entries",
        r#"["target", "vendor/cache/", "docs/plans"]"#,
    );
    assert_eq!(err, refusal("vendor/cache/", "cache"));
}

/// §FS-config.3.5.16: an entry with no non-empty component is offered removal
/// alone, so its message ends at the first repair.
#[test]
fn scan_exclude_offers_only_removal_for_a_bare_slash() {
    let err = load_err(
        "scan_exclude_offers_only_removal_for_a_bare_slash",
        r#"["/"]"#,
    );
    assert_eq!(
        err,
        "grund.toml:4: [scan] exclude entry `/` contains `/`, not a directory name; \
         remove it to keep the scan unchanged"
    );
}

/// §FS-config.3.5.16: only `/` is refused — an empty list and a list of names
/// load unchanged.
#[test]
fn scan_exclude_accepts_names_and_an_empty_list() {
    let empty = load_exclude("scan_exclude_accepts_an_empty_list", "[]").expect("empty list");
    assert!(empty.is_empty());
    let names = load_exclude(
        "scan_exclude_accepts_names",
        r#"["target", "plans", "node_modules"]"#,
    )
    .expect("names load");
    assert_eq!(names, ["target", "plans", "node_modules"]);
}
