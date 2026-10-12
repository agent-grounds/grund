//! The trees of `command_agreement.rs` for what a stub answers: the rows of
//! §REQ-one-answer.differences.stubs, each around the stub `docs/x.md` to a
//! target `src/x.rs` outside `[scan] include`, asked of the command line, of shell
//! completion and of the editor snapshot. U01 and U05 closed with #597, so their
//! trees stay here to agree (§FS-declarations.stubs.target.5, §FS-rules.7.10).

use super::agreement_tree::{Tree, Verdict, agree};
use grund_core::{LspSnapshotOpts, lsp_snapshot};
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

/// Closed row U01: a citation into a section the outside target declares twice is
/// refused by `check` as `show` refuses it (§REQ-one-answer.stubs.outside-include),
/// `check` reporting the section where the target declares it twice
/// (§FS-declarations.checks.duplicate-section.2).
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

/// `src/x.rs`, declaring `FS-x` with the section `1`, outside the include.
fn section_tree(name: &str) -> Tree {
    stub_tree(name, CONFIG, TARGET_SECTION, &[])
}

/// Row U03b: shell completion offers the outside target's section a direct `show`
/// returns (§REQ-one-answer.stubs.outside-include).
pub fn completion_offers_the_targets_section() -> Verdict {
    let tree = section_tree("u03b");
    let offered =
        super::agreement_tree::stdout(&tree.grund(&["complete", "ids", "--prefix", "FS-x."]))
            .lines()
            .any(|line| line == "FS-x.1");
    agree(
        "complete ids offers FS-x.1 / show FS-x.1 answers",
        offered,
        tree.code(&["FS-x.1"]) == 0,
    )
}

/// Row U03c: the editor snapshot enumerates the outside target's section a direct
/// `show` returns, so completion and navigation reach it
/// (§REQ-one-answer.stubs.outside-include).
pub fn the_editor_enumerates_the_targets_section() -> Verdict {
    let tree = section_tree("u03c");
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: tree.0.clone(),
        path_provided: true,
        ..LspSnapshotOpts::default()
    })
    .expect("editor snapshot");
    let enumerated = snapshot
        .sections
        .iter()
        .any(|section| section.query_id == "FS-x.1");
    agree(
        "the editor snapshot enumerates FS-x.1 / show FS-x.1 answers",
        enumerated,
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

/// Closed row U05: a RULE target's title that `show --brief` reads is one `check`
/// reads (§REQ-one-answer.stubs.verdict, §FS-rules.7.10), outside the include as
/// with the target scanned.
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

/// The U04 tree's `FS` and `GOAL` kinds under `docs/fs` and `docs/goals`, with
/// named sections, and `include` as given.
fn goal_config(include: &str) -> String {
    CONFIG
        .replace("folder = \"docs\"", "folder = \"docs/fs\"")
        .replace("[id]\n", "[id]\nnamed_sections = true\n")
        .replace(
            "[scan]\n",
            "[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n[scan]\n",
        )
        .replace("include = [\"docs\"]", include)
}

/// The stub `docs/fs/x.md` to a `src/x.rs` that cites `GOAL-g` in its lead and
/// declares a Terms chapter, with `src/` outside the include or scanned.
fn goal_tree(name: &str, include: &str) -> Tree {
    let config = goal_config(include);
    Tree::new(
        name,
        &[
            ("grund.toml", config.as_str()),
            ("docs/fs/x.md", "# FS-x: [../../src/x.rs](../../src/x.rs)\n"),
            ("docs/goals/g.md", "# GOAL-g: G\n\nLead.\n"),
            (
                "src/x.rs",
                "/// FS-x: X\n///\n/// Lead, for \u{a7}GOAL-g.\n///\n/// ## terms: Terms\n///\n\
                 /// Words.\npub fn x() {}\n",
            ),
        ],
    )
}

/// The trees of a row asked outside the include and with the target scanned.
const OUTSIDE: &str = "include = [\"docs\"]";
const SCANNED: &str = "include = [\"docs\", \"src\"]";

/// Row U07: a chapter rule counts an outside target's chapters as it counts them
/// with the target scanned (§REQ-one-answer.stubs.outside-include).
pub fn a_chapter_rule_counts_the_targets_chapters() -> Verdict {
    let codes = |name: &str, include: &str| {
        goal_tree(name, include)
            .json(&[
                "check",
                "--rule",
                "Each FS must have exactly one Terms chapter.",
                "--only-rule",
                "--format",
                "json",
            ])
            .iter()
            .map(|row| row["code"].as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>()
    };
    agree(
        "the chapter rule's findings, target outside / target scanned",
        codes("u07-outside", OUTSIDE),
        codes("u07-scanned", SCANNED),
    )
}

/// Row U08: `refs` lists an outside target's citation as it lists it with the
/// target scanned (§REQ-one-answer.stubs.outside-include).
pub fn refs_lists_the_targets_citation() -> Verdict {
    let sites = |name: &str, include: &str| {
        goal_tree(name, include)
            .json(&["refs", "GOAL-g", "--format", "json"])
            .iter()
            .map(site)
            .collect::<Vec<_>>()
    };
    agree(
        "refs GOAL-g, target outside / target scanned",
        sites("u08-outside", OUTSIDE),
        sites("u08-scanned", SCANNED),
    )
}
