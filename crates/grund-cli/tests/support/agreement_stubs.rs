//! The trees of `command_agreement.rs` for what a stub answers: the rows of
//! §REQ-one-answer.differences.stubs the command line can show, each around the
//! stub `docs/x.md` to a target `src/x.rs` outside `[scan] include`.

use super::agreement_tree::{Tree, Verdict, agree};
use serde_json::Value;

/// `FS` in `docs/`, the include `docs` alone, so `src/` is outside it.
const CONFIG: &str = "grund_config_version = 1\nproject_name = \"fixture\"\n\
    [reference]\nstrict = true\nrequire_grounding = false\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n[scan]\ninclude = [\"docs\"]\n";
const STUB: &str = "# FS-x: [../src/x.rs](../src/x.rs)\n";
const TARGET: &str = "/// FS-x: X\n///\n/// Lead.\npub fn x() {}\n";
const TARGET_SECTION: &str =
    "/// FS-x: X\n///\n/// Lead.\n///\n/// ## 1. One\n///\n/// First.\npub fn x() {}\n";

/// The stub tree with `target` behind it and `extra` files beside it.
fn stub_tree(name: &str, config: &str, target: &str, extra: &[(&str, &str)]) -> Tree {
    let files = [
        ("grund.toml", config),
        ("docs/x.md", STUB),
        ("src/x.rs", target),
    ];
    Tree::new(name, &[&files[..], extra].concat())
}

/// `path:line` of a JSON row, or `""`.
fn site(row: &Value) -> String {
    format!("{}:{}", row["path"].as_str().unwrap_or(""), row["line"])
}

/// `path:line` of `id`'s own row (no section) among one command's JSON rows.
fn site_of(tree: &Tree, args: &[&str], id: &str) -> String {
    tree.json(args)
        .iter()
        .find(|row| row["id"] == id && row["section"].is_null())
        .map(site)
        .unwrap_or_default()
}

/// Row Placement: a healthy stub's declaration is at one site for `list` and
/// `list --size` (§REQ-one-answer.stubs.outside-include).
pub fn a_stubbed_declaration_is_at_one_site() -> Verdict {
    let tree = stub_tree("placement", CONFIG, TARGET, &[]);
    agree(
        "list's FS-x / list --size's FS-x",
        site_of(&tree, &["list", "--format", "json"], "FS-x"),
        site_of(&tree, &["list", "--size", "--format", "json"], "FS-x"),
    )
}

/// Row U01: a citation into a section the outside target declares twice is
/// refused by `check` as `show` refuses it (§REQ-one-answer.stubs.outside-include).
pub fn a_twice_declared_section_is_refused_once() -> Verdict {
    let target = concat!(
        "/// FS-x: X\n///\n/// Lead.\n///\n/// ## 1. One\n///\n/// First.\n///\n",
        "/// ## 1. One again\n///\n/// Second.\npub fn x() {}\n",
    );
    let tree = stub_tree(
        "u01",
        CONFIG,
        target,
        &[("docs/uses.md", "Uses \u{a7}FS-x.1.\n")],
    );
    agree(
        "check passes / show FS-x.1 answers",
        tree.code(&["check"]) == 0,
        tree.code(&["FS-x.1"]) == 0,
    )
}

/// Row U03a: exhaustive `show --batch` returns the outside target's section a
/// direct `show` returns (§REQ-one-answer.stubs.outside-include).
pub fn the_batch_returns_the_targets_section() -> Verdict {
    let tree = stub_tree("u03a", CONFIG, TARGET_SECTION, &[]);
    let batched = tree
        .json(&["show", "--batch", "--all", "--format", "json"])
        .iter()
        .any(|row| {
            row["query"]["id"] == "FS-x" && row["query"]["section"] == "1" && row["ok"] == true
        });
    agree(
        "show --batch --all returns FS-x.1 / show FS-x.1 answers",
        batched,
        tree.code(&["FS-x.1"]) == 0,
    )
}

/// Row U03d: `list --selector FS.terms` selects the outside target's chapter a
/// direct `show` returns (§REQ-one-answer.stubs.outside-include).
pub fn the_selector_selects_the_targets_chapter() -> Verdict {
    let config = CONFIG.replace("[id]\n", "[id]\nnamed_sections = true\n");
    let target =
        "/// FS-x: X\n///\n/// Lead.\n///\n/// ## terms: Terms\n///\n/// Words.\npub fn x() {}\n";
    let tree = stub_tree("u03d", &config, target, &[]);
    let selected = tree
        .json(&["list", "--selector", "FS.terms", "--format", "json"])
        .iter()
        .any(|row| row["id"] == "FS-x" && row["section"] == "terms");
    agree(
        "list --selector FS.terms selects FS-x.terms / show FS-x.terms answers",
        selected,
        tree.code(&["FS-x.terms"]) == 0,
    )
}

/// Row U04: a citation obligation is judged on an outside target as on the same
/// target scanned (§REQ-one-answer.stubs.outside-include).
pub fn an_obligation_is_judged_on_the_target() -> Verdict {
    let config = |include: &str| {
        CONFIG
            .replace("folder = \"docs\"", "folder = \"docs/fs\"")
            .replace(
                "[scan]\n",
                "[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n[scan]\n",
            )
            .replace("include = [\"docs\"]", include)
    };
    let findings = |name: &str, include: &str| {
        let config = config(include);
        let tree = Tree::new(
            name,
            &[
                ("grund.toml", config.as_str()),
                ("docs/fs/x.md", "# FS-x: [../../src/x.rs](../../src/x.rs)\n"),
                ("docs/goals/g.md", "# GOAL-g: G\n\nLead.\n"),
                (
                    "src/x.rs",
                    "/// FS-x: X\n///\n/// Lead, for \u{a7}GOAL-g.\npub fn x() {}\n",
                ),
            ],
        );
        tree.json(&[
            "check",
            "--rule",
            "Each FS must not cite any GOAL.",
            "--only-rule",
            "--format",
            "json",
        ])
        .iter()
        .map(|row| row["code"].as_str().unwrap_or("").to_string())
        .collect::<Vec<_>>()
    };
    agree(
        "the obligation's findings, target outside / target scanned",
        findings("u04-outside", "include = [\"docs\"]"),
        findings("u04-scanned", "include = [\"docs\", \"src\"]"),
    )
}

/// Row U05: a RULE target's title that `show --brief` reads is one `check` reads
/// (§REQ-one-answer.stubs.verdict). The tree shows it outside the include; the
/// same holds for any RULE behind a stub today.
pub fn a_rule_targets_title_is_read_once() -> Verdict {
    let config = CONFIG.replace("folder = \"docs\"", "folder = \"docs/fs\"").replace(
        "[scan]\n",
        "[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n[scan]\n",
    );
    let tree = Tree::new(
        "u05-rule-title",
        &[
            ("grund.toml", config.as_str()),
            (
                "docs/rules/r.md",
                "# RULE-r: [../../src/r.rs](../../src/r.rs)\n",
            ),
            (
                "src/r.rs",
                "/// RULE-r: Each FS must not cite any FS.\n///\n/// Lead.\npub fn r() {}\n",
            ),
        ],
    );
    let untitled = tree.json(&["check", "--format", "json"]).iter().any(|row| {
        row["message"]
            .as_str()
            .is_some_and(|m| m.contains("has no title"))
    });
    let brief = tree.json(&["RULE-r", "--brief", "--format", "json"]);
    let titled = brief
        .first()
        .and_then(|row| row["body"].as_str())
        .is_some_and(|body| {
            body.lines()
                .next()
                .is_some_and(|line| line.contains("Each FS must not cite any FS."))
        });
    agree(
        "check finds a title / show --brief reads one",
        !untitled,
        titled,
    )
}

/// Row U06: the declaration `check --only unused` places is the one `show` places
/// (§REQ-one-answer.stubs.outside-include).
pub fn an_unused_declaration_is_at_one_site() -> Verdict {
    let tree = stub_tree("u06", CONFIG, TARGET, &[]);
    let unused = tree
        .json(&["check", "--only", "unused", "--format", "json"])
        .iter()
        .find(|row| row["code"] == "unused")
        .map(site)
        .unwrap_or_default();
    agree(
        "check --only unused / show FS-x",
        unused,
        site_of(&tree, &["FS-x", "--format", "json"], "FS-x"),
    )
}
