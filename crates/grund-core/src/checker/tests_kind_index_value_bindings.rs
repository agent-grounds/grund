//! Test module: what of a folder-owned ID's citations in its own index is the
//! entry (§FS-check.4.1.2.1). Only a citation of the declaration itself is
//! navigation; a section citation or a value binding's citation on the same page
//! is use, for `check` and `list --unused` alike (agent-grounds/grund#645).

use std::path::{Path, PathBuf};

use crate::api::{ListOpts, list};
use crate::testing::{check_run, findings, test_root, write};

const COMP_CONFIG: &str = "grund_config_version = 1\n\n\
     [reference]\nstrict = true\n\n\
     [id]\nnamed_sections = true\nformat = \"{kind}-{slug}\"\n\n\
     [[kinds]]\nkind = \"COMP\"\nfolder = \"docs/competition\"\nvalue_chapter = \"values\"\n\n\
     [scan]\ninclude = [\"docs\"]\n";

const COMP_DECLARATION: &str = "# COMP-zz-a: Competitor A\n\n\
     ## values: Values\n\n\
     ### values.price: Price\n\n\
     #### values.price.1: 30\n\n\
     #### values.price.2: USD\n\n\
     #### values.price.3: contributor\n\n\
     #### values.price.4: month\n";

const ENTRY: &str = "- [§COMP-zz-a](COMP-zz-a.md#comp-zz-a-competitor-a)\n";

const BINDING: &str = "Price: `30 USD contributor month` (§COMP-zz-a.values.price)\n";

/// The issue's fixture: a `COMP` kind whose index holds the entry for
/// `COMP-zz-a`, followed by `index_tail`; `roadmap` is a page outside the folder.
fn comp_repo(name: &str, index_tail: &str, roadmap: &str) -> PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), COMP_CONFIG);
    write(
        &root.join("docs/competition/COMP-zz-a.md"),
        COMP_DECLARATION,
    );
    write(
        &root.join("docs/competition/README.md"),
        &format!("# Competition\n\n{ENTRY}\n{index_tail}"),
    );
    write(
        &root.join("docs/roadmap.md"),
        &format!("# Roadmap\n\n{roadmap}"),
    );
    root
}

fn unused_ids(root: &Path) -> Vec<String> {
    let run = check_run(root, false);
    assert!(
        run.report.errors.is_empty(),
        "the fixture is otherwise clean: {:?}",
        findings(&run)
    );
    run.report
        .warnings
        .iter()
        .filter(|diagnostic| diagnostic.code == "unused")
        .filter_map(|diagnostic| {
            diagnostic
                .message
                .strip_prefix("declared but never cited: ")
        })
        .map(|tail| tail.split([' ', ';']).next().unwrap().to_string())
        .collect()
}

/// §FS-check.4.1.2.1 / §FS-values.3.2: a value binding in the kind's own index
/// is use, beside the entry that is not.
#[test]
fn a_value_binding_in_its_own_index_counts_as_use() {
    let root = comp_repo(
        "a_value_binding_in_its_own_index_counts_as_use",
        BINDING,
        "",
    );
    let unused = unused_ids(&root);
    assert!(
        !unused.contains(&"COMP-zz-a".to_string()),
        "the binding is use, not the entry: {unused:?}"
    );
}

/// §FS-check.4.1.3: the control — the same binding on a page outside the folder.
#[test]
fn the_same_value_binding_outside_the_index_counts_as_use() {
    let root = comp_repo(
        "the_same_value_binding_outside_the_index_counts_as_use",
        "",
        BINDING,
    );
    let unused = unused_ids(&root);
    assert!(
        !unused.contains(&"COMP-zz-a".to_string()),
        "a binding on another page is use: {unused:?}"
    );
}

/// §FS-check.4.1.2: the exclusion survives — an index holding only the entry
/// leaves the declaration uncited.
#[test]
fn an_index_holding_only_the_entry_still_leaves_the_declaration_unused() {
    let root = comp_repo(
        "an_index_holding_only_the_entry_still_leaves_the_declaration_unused",
        "",
        "",
    );
    let unused = unused_ids(&root);
    assert!(
        unused.contains(&"COMP-zz-a".to_string()),
        "the entry is navigation, not use: {unused:?}"
    );
}

/// §FS-check.4.1.2.1: a plain section citation of a folder-owned ID in its
/// index, not a binding, names a section and so is not the entry.
#[test]
fn a_section_citation_in_its_own_index_counts_as_use() {
    let root = comp_repo(
        "a_section_citation_in_its_own_index_counts_as_use",
        "Prices are kept in §COMP-zz-a.values.price.\n",
        "",
    );
    let unused = unused_ids(&root);
    assert!(
        !unused.contains(&"COMP-zz-a".to_string()),
        "a section citation is use, not the entry: {unused:?}"
    );
}

/// §FS-check.4.1.2.1 / §FS-values.3.1.2: a binding aimed at a whole-declaration
/// value root names no section, and is still not the entry.
#[test]
fn a_root_aimed_binding_without_a_section_in_its_own_index_counts_as_use() {
    let root = test_root("a_root_aimed_binding_without_a_section_in_its_own_index_counts_as_use");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nvalues = true\n\n\
         [scan]\ninclude = [\"values\"]\nextensions = [\"md\"]\n",
    );
    write(
        &root.join("values/quota.md"),
        "# CONST-quota: Yearly quota\n## 1. 1200\n## 2. USD\n",
    );
    write(
        &root.join("values/README.md"),
        "# Values\n\n- [§CONST-quota](quota.md#const-quota-yearly-quota)\n\n\
         Quota: `1200 USD` (§CONST-quota)\n",
    );
    let unused = unused_ids(&root);
    assert!(
        !unused.contains(&"CONST-quota".to_string()),
        "a root-aimed binding is use, not the entry: {unused:?}"
    );
}

/// §FS-list.1.3: `list --unused` discounts the entry within the same bounds as
/// `check`, so the binding in the index keeps the declaration off the list.
#[test]
fn list_unused_counts_a_value_binding_in_its_own_index() {
    let root = comp_repo(
        "list_unused_counts_a_value_binding_in_its_own_index",
        BINDING,
        "",
    );
    let output = list(ListOpts {
        path: root.clone(),
        path_provided: true,
        unused_only: true,
        ..ListOpts::default()
    })
    .expect("list the COMP fixture");
    let ids: Vec<&str> = output
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    assert!(
        !ids.contains(&"COMP-zz-a"),
        "list --unused agrees with check: {ids:?}"
    );
}
