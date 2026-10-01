//! Root-aimed value bindings (§FS-values.3.1.2, §FS-values.5.1,
//! §FS-values.5.2.2): the join and the split over every authority, the rule that
//! gives each path one reading, the walk that stops at the first unequal
//! component, and the bare citation of a declaration that is not a value, which
//! stays prose (§FS-values.3.1.1).

use std::path::{Path, PathBuf};

use crate::model::{
    authored_component, first_unequal_component, joined_value_components, root_literal_parts,
};
use crate::testing::{check_run, test_root, write};

/// One repository holding a root of every authority — a Markdown and a JSON
/// whole value, a marked root and a chapter root, each with a component that
/// carries a no-break space — plus an ordinary `NOTE` declaration. `uses` lands
/// at line 9 of `docs/offer.md`.
fn roots_repo(name: &str, uses: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [[kinds]]\nkind = \"DOC\"\nfolder = \"docs\"\nindex = false\nvalue_chapter = \"values\"\n\n\
         [[kinds]]\nkind = \"NOTE\"\nfolder = \"notes\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\", \"notes\"]\nextensions = [\"md\"]\n",
    );
    write(
        &root.join("values/quota.md"),
        "# CONST-quota: Yearly quota\n## 1. 1200\n## 2. USD\n## 3. per\u{a0}year\n",
    );
    write(
        &root.join("values/runtime.json"),
        "{\n  \"CONST-limit\": [25, \"per\\u00a0hour\"]\n}\n",
    );
    write(
        &root.join("notes/plain.md"),
        "# NOTE-plain: Plain note\n\n## 1. 1200 USD\n",
    );
    write(
        &root.join("docs/offer.md"),
        &format!(
            "# DOC-offer: Offer\n\n\
             ## 1. Quota <!-- grund:value -->\n\
             ### 1.1. 1200\n\
             ### 1.2. per\u{a0}year\n\n\
             ## 2. Use\n\n\
             {uses}\n\
             ## values: Values\n\n\
             ### values.rate: Rate\n\n\
             #### values.rate.1: 3\n\
             #### values.rate.2: per\u{a0}day\n"
        ),
    );
    root
}

/// Every error the check reports, as `code: message`.
fn errors(root: &Path) -> Vec<String> {
    check_run(root, false)
        .report
        .errors
        .into_iter()
        .map(|error| format!("{}: {}", error.code, error.message))
        .collect()
}

fn decoded(components: &[crate::model::ValueComponent]) -> Vec<&str> {
    components
        .iter()
        .map(|component| component.decoded.as_str())
        .collect()
}

/// Joining a run with one ASCII space and splitting the literal back gives the
/// run again, because only U+0020 separates: a no-break space stays inside its
/// part. Two adjacent spaces keep the empty part between them
/// (§FS-values.3.1.2).
#[test]
fn the_join_splits_back_into_its_components_at_ascii_space_alone() {
    let run = ["1200", "USD", "per\u{a0}year"].map(|text| authored_component(text, 1));
    let run = run.iter().collect::<Vec<_>>();
    let joined = joined_value_components(&run);
    assert_eq!(joined, "1200 USD per\u{a0}year");
    let parts = root_literal_parts(&authored_component(&joined, 1));
    assert_eq!(decoded(&parts), ["1200", "USD", "per\u{a0}year"]);
    assert_eq!(first_unequal_component(&parts, &run), None);

    let doubled = root_literal_parts(&authored_component("1200  USD", 1));
    assert_eq!(decoded(&doubled), ["1200", "", "USD"]);
}

/// The same round trip holds under every authority a root can come from, each
/// component keeping its own equality, so a respelled number still agrees
/// (§FS-values.3.1.2, §FS-values.4).
#[test]
fn a_root_literal_agrees_under_every_authority() {
    let root = roots_repo(
        "value_roots_agree_everywhere",
        "`1200.0 USD per\u{a0}year` (§CONST-quota)\n\n\
         `2.5e1 per\u{a0}hour` (§CONST-limit)\n\n\
         `1.2e3 per\u{a0}year` (§DOC-offer.1)\n\n\
         `3.0 per\u{a0}day` (§DOC-offer.values.rate)\n",
    );
    assert_eq!(errors(&root), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(root);
}

/// The walk compares in declared order and stops at the first inequality, so a
/// literal wrong in two components reports the earlier one alone; a literal that
/// joins with a no-break space has fewer parts than the run, so it names the
/// root (§FS-values.5.2.2).
#[test]
fn the_walk_stops_at_the_first_unequal_component() {
    let run = ["1200", "USD", "per\u{a0}year"].map(|text| authored_component(text, 1));
    let run = run.iter().collect::<Vec<_>>();
    let parts = root_literal_parts(&authored_component("1200.0 EUR per\u{a0}month", 1));
    assert_eq!(first_unequal_component(&parts, &run), Some(1));

    let root = roots_repo(
        "value_roots_first_unequal",
        "`1200.0 EUR per\u{a0}month` (§CONST-quota)\n\n\
         `1200\u{a0}USD per\u{a0}year` (§CONST-quota)\n",
    );
    assert_eq!(
        errors(&root),
        [
            "value-mismatch: value mismatch for CONST-quota.2: bound `EUR`, declared `USD` \
             at values/quota.md:3",
            "value-mismatch: value mismatch for CONST-quota: bound `1200\u{a0}USD per\u{a0}year`, \
             declared `1200 USD per\u{a0}year` at values/quota.md:2",
        ]
    );
    let _ = std::fs::remove_dir_all(root);
}

/// No path reads both ways over valid authority (§FS-values.5.1). Where a path
/// would be one root and another root's component — a mark inside a whole value,
/// a mark nested inside a mark — the declaration is invalid, so neither reading
/// compares: a literal unequal to everything there earns no mismatch.
#[test]
fn no_path_reads_both_ways_over_valid_authority() {
    let root = test_root("value_roots_one_reading");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [[kinds]]\nkind = \"DOC\"\nfolder = \"docs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    write(
        &root.join("values/dual.md"),
        "# CONST-dual: Dual\n## 1. 5 <!-- grund:value -->\n### 1.1. 6\n",
    );
    write(
        &root.join("docs/nest.md"),
        "# DOC-nest: Nest\n\n\
         ## 1. Outer <!-- grund:value -->\n\
         ### 1.1. Inner <!-- grund:value -->\n\
         #### 1.1.1. 7\n\n\
         ## 2. Use\n\n\
         `999` (§CONST-dual.1)\n\n\
         `999` (§CONST-dual)\n\n\
         `999` (§DOC-nest.1.1)\n\n\
         `999 999` (§DOC-nest.1)\n",
    );
    let errors = errors(&root);
    assert!(
        !errors
            .iter()
            .any(|error| error.starts_with("value-mismatch") || error.contains("bound at its root")),
        "an overlap compares neither way: {errors:?}"
    );
    for id in ["CONST-dual", "DOC-nest"] {
        assert!(
            errors.iter().any(|error| error.starts_with(&format!(
                "invalid-value-declaration: invalid value declaration for {id}"
            ))),
            "the overlap is the declaration's error: {errors:?}"
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

/// A delimited form aimed at a declaration that is not a value — an ordinary
/// one, or one that only holds marked roots — or at an ordinary section, binds
/// nothing and reports nothing, now that a bare ID is binding grammar
/// (§FS-values.3.1.1, §FS-values.3.1.2).
#[test]
fn a_delimited_citation_of_an_ordinary_declaration_stays_prose() {
    let root = roots_repo(
        "value_roots_ordinary_prose",
        "`1200 USD` (§NOTE-plain)\n\n\
         `1200 USD` (§NOTE-plain.1)\n\n\
         `999` (§NOTE-plain)\n\n\
         `999` (§DOC-offer)\n",
    );
    assert_eq!(errors(&root), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(root);
}
