//! Black-box: each home rule judges each spelling at its own path, and a value source linked
//! into a second home names both spellings once (§FS-scan.identity.findings.place,
//! §FS-values.2.3, §FS-declarations.checks.misplaced-declaration.2,
//! §FS-config-v2.schema.places.ownership, §FS-config.3.9.2). This is #1216's reproducer.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

fn value_tree(case: &str, home: &str, suffix: &str) -> Tree {
    let tree = Tree::new(
        case,
        &format!(
            "grund_config_version = 2\nproject_name = \"values\"\n\
             [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\n\
             [schema.kinds.OTHER]\nfolders = [\"{home}\"]\nform = \"value\"\n\n\
             [schema.kinds.page]\ncitable = false\nfiles = [\"docs/offer.md\"]\n"
        ),
    );
    let content = if suffix == "json" {
        "{\"CONST-room-limit\": [40, \"rooms\"]}\n"
    } else {
        "CONST-room-limit: [40, rooms]\n"
    };
    let index = format!("# Values\n\n- [\u{a7}CONST-room-limit](limits.{suffix})\n");
    tree.put(&format!("values/limits.{suffix}"), content);
    tree.put("values/README.md", &index);
    tree.put(&format!("{home}/README.md"), &index);
    tree.put(
        "docs/offer.md",
        "# Offer\n\nThe limit is `40` (\u{a7}CONST-room-limit.1).\n",
    );
    tree.link(
        &format!("{home}/limits.{suffix}"),
        format!("../values/limits.{suffix}"),
    );
    tree
}

/// The cases #1216 names: JSON and YAML, the second home sorting before and after `values`.
const CASES: [(&str, &str); 4] = [("a", "json"), ("z", "json"), ("a", "yaml"), ("z", "yaml")];

fn misplaced(home: &str, suffix: &str) -> String {
    format!(
        "{home}/limits.{suffix}:1: error: CONST-room-limit declares kind CONST inside OTHER home {home}\n"
    )
}

/// The one duplicate line, located at the first site and naming the other.
fn duplicate(home: &str, suffix: &str) -> String {
    let (first, other) = if home < "values" {
        (
            format!("{home}/limits.{suffix}"),
            format!("values/limits.{suffix}"),
        )
    } else {
        (
            format!("values/limits.{suffix}"),
            format!("{home}/limits.{suffix}"),
        )
    };
    format!(
        "{first}:1: error: duplicate declaration of CONST-room-limit (also declared at {other}:1)\n"
    )
}

fn both_lines(home: &str, suffix: &str) -> String {
    let mut lines = [misplaced(home, suffix), duplicate(home, suffix)];
    lines.sort();
    lines.concat()
}

#[test]
fn a_value_source_in_two_homes_names_each_spelling_once_whichever_sorts_first() {
    for (home, suffix) in CASES {
        let tree = value_tree(&format!("whole-{home}-{suffix}"), home, suffix);
        let check = tree.grund(&["check"]);
        assert_eq!(check.stdout, both_lines(home, suffix));
        assert_eq!(check.code, 1);
    }
}

/// #1216's JSON `sites` names both spellings (§FS-errors.5.1).
#[test]
fn its_json_sites_name_both_spellings() {
    for (home, suffix) in CASES {
        let tree = value_tree(&format!("sites-{home}-{suffix}"), home, suffix);
        let json = tree.grund(&["check", "--format", "json"]);
        let mut want = vec![
            format!("{home}/limits.{suffix}:1"),
            format!("values/limits.{suffix}:1"),
        ];
        want.sort();
        assert_eq!(json.sites("duplicate"), want);
        let misplaced = json
            .findings()
            .iter()
            .filter(|row| row["code"] == "misplaced-declaration")
            .count();
        assert_eq!(misplaced, 1, "{}", json.stdout);
    }
}

/// #1216's `show`: the ambiguous line names both spellings.
#[test]
fn show_of_it_names_both_spellings() {
    for (home, suffix) in CASES {
        let tree = value_tree(&format!("show-{home}-{suffix}"), home, suffix);
        let mut sites = [
            format!("{home}/limits.{suffix}:1"),
            format!("values/limits.{suffix}:1"),
        ];
        sites.sort();
        let show = tree.grund(&["show", "CONST-room-limit"]);
        assert_eq!(show.code, 1);
        let want = format!(
            "ambiguous ID: CONST-room-limit (declared at {}, {})",
            sites[0], sites[1]
        );
        assert!(
            show.stderr.contains(&want) || show.stdout.contains(&want),
            "{}{}",
            show.stdout,
            show.stderr
        );
    }
}

/// The scoped table: a finding that names several sites is in scope at any of them and keeps its
/// bytes; the misplaced line belongs to the link's home alone (§FS-check.1.3.6.2).
#[test]
fn a_run_naming_either_spelling_keeps_the_duplicate() {
    for (home, suffix) in CASES {
        let tree = value_tree(&format!("scoped-{home}-{suffix}"), home, suffix);
        let link = tree.grund(&["check", &format!("{home}/limits.{suffix}")]);
        assert_eq!(link.stdout, both_lines(home, suffix));
        assert_eq!(link.code, 1);
        let real = tree.grund(&["check", &format!("values/limits.{suffix}")]);
        assert_eq!(real.stdout, duplicate(home, suffix));
        assert_eq!(real.code, 1);
    }
}

fn text_tree(case: &str, config: &str, id: &str, home: &str, wrong: &str) -> Tree {
    let tree = Tree::new(case, config);
    tree.put(
        &format!("{home}/x.md"),
        &format!("# {id}: X\n\nCites \u{a7}{id}.\n"),
    );
    tree.link(&format!("{wrong}/x.md"), format!("../{}/x.md", &home[5..]));
    tree
}

fn wrong_home(id: &str, wrong: &str) -> String {
    format!("{wrong}/x.md:1: error: {id} declares kind FS inside AR home {wrong}\n")
}

/// Whichever directory sorts first: the link in the home that sorts before the right one, and
/// after it. Either way one error at the link and none at the real name.
const ORDERS: [(&str, &str); 2] = [("docs/fs", "docs/ar"), ("docs/aa", "docs/zz")];

/// The owner's addition: a Markdown file in two homes gets one error at the wrong spelling and
/// none at the right one.
#[test]
fn a_text_file_in_two_homes_is_judged_at_each_spelling_under_version_2() {
    for (home, wrong) in ORDERS {
        let config = format!(
            "grund_config_version = 2\nproject_name = \"t\"\n\
             [schema.kinds.FS]\nfolders = [\"{home}\"]\nindex = false\n\n\
             [schema.kinds.AR]\nfolders = [\"{wrong}\"]\nindex = false\n"
        );
        let case = format!("text-v2-{}", &wrong[5..]);
        let check = text_tree(&case, &config, "FS-x", home, wrong).grund(&["check"]);
        assert_eq!(
            check.stdout,
            wrong_home("FS-x", wrong),
            "the link in {wrong}"
        );
        assert_eq!(check.code, 1);
    }
}

/// "A file" in the home rules means the spelling in v1 too.
#[test]
fn a_text_file_in_two_homes_is_judged_at_each_spelling_under_version_1() {
    for (home, wrong) in ORDERS {
        let config = format!(
            "project_name = \"t\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"{home}\"\nindex = false\n\n\
             [[kinds]]\nkind = \"AR\"\nfolder = \"{wrong}\"\nindex = false\n"
        );
        let case = format!("text-v1-{}", &wrong[5..]);
        let check = text_tree(&case, &config, "FS-001-x", home, wrong).grund(&["check"]);
        assert_eq!(
            check.stdout,
            wrong_home("FS-001-x", wrong),
            "the link in {wrong}"
        );
        assert_eq!(check.code, 1);
    }
}

/// A value source with two names in one home has one owner: no duplicate, and its finding names
/// both (§FS-values.2.3, §FS-scan.identity.findings.content).
#[test]
fn a_value_source_with_two_names_in_one_home_is_no_duplicate_and_its_finding_names_both() {
    let tree = Tree::new(
        "one-home",
        "grund_config_version = 2\nproject_name = \"o\"\n\
         [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n",
    );
    tree.put("values/limits.json", "{\"CONST-a\": {\"x\": 1}}\n");
    tree.link("values/z.json", "limits.json");
    let json = tree.grund(&["check", "--format", "json"]);
    assert!(!json.stdout.contains("\"duplicate\""), "{}", json.stdout);
    assert_eq!(
        json.sites("invalid-value-declaration"),
        ["values/limits.json:1", "values/z.json:1"]
    );
    let text = tree.grund(&["check"]);
    assert!(
        text.stdout.contains("(also at values/z.json:1)"),
        "{}",
        text.stdout
    );
}

/// The contrast the spec draws: the same second name means three things by what the file is.
#[test]
fn a_second_home_means_a_misplaced_text_file_but_a_duplicate_value_source() {
    let head = "grund_config_version = 2\nproject_name = \"t\"\n";
    let text = text_tree(
        "contrast-text",
        &format!(
            "{head}[schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\n\
             [schema.kinds.AR]\nfolders = [\"docs/ar\"]\nindex = false\n"
        ),
        "FS-x",
        "docs/fs",
        "docs/ar",
    );
    let codes = |tree: &Tree| -> Vec<String> {
        let run = tree.grund(&["check", "--format", "json"]);
        let mut codes: Vec<String> = run
            .findings()
            .iter()
            .filter(|row| row["severity"] == "error")
            .map(|row| row["code"].as_str().unwrap().to_string())
            .collect();
        codes.sort();
        codes
    };
    assert_eq!(codes(&text), ["misplaced-declaration"]);
    let value = value_tree("contrast-value", "a", "json");
    assert_eq!(codes(&value), ["duplicate", "misplaced-declaration"]);
}

/// Guard: a single-file kind compares physically, so a link to its document from elsewhere is
/// that document, not a second one (§FS-declarations.checks.misplaced-declaration.1).
#[test]
fn a_link_to_a_single_file_home_is_that_document() {
    let tree = Tree::new(
        "single-file",
        "grund_config_version = 2\nproject_name = \"g\"\n\
         [schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n",
    );
    tree.put("docs/goals.md", "# GOAL-a: A\n\nCites \u{a7}GOAL-a.\n");
    tree.link("a/alias.md", "../docs/goals.md");
    let check = tree.grund(&["check"]);
    assert!(
        !check.stdout.contains("must be declared"),
        "{}",
        check.stdout
    );
    assert_eq!(check.code, 0, "{}", check.stdout);
}
