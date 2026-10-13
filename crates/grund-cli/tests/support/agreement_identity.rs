//! The trees of `command_agreement.rs` for which declarations an ID has and what
//! a narrowed run resolves: the reproduced rows r1 and r2 of #576, and the rows
//! of §REQ-one-answer.differences.identity and §REQ-one-answer.differences.scope.

use super::agreement_tree::{
    CONFIG_NUMBER, CONFIG_SLUG, Tree, Verdict, agree, json_lines, stderr, stdout,
};

/// A workspace whose root kind is `FS` and whose member `api` declares `API-a`.
const WORKSPACE: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n[workspace]\nmembers = [\"api\"]\n";
const MEMBER: &str = "grund_config_version = 1\nproject_name = \"api\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"API\"\nfile = \"docs/api.md\"\n";
const CITES_MEMBER: &str = "# FS-a: A\n\nCites \u{a7}api/API-a.\n";
const MEMBER_DOC: &str = "# API-a: A\n\nThe interface.\n";

/// What a refusal names as failed: its first `error:` line, up to the offer.
fn reason(line: &str) -> String {
    let line = line.strip_prefix("error: ").unwrap_or(line);
    line.split(';').next().unwrap_or(line).to_string()
}

/// r1, row G01: a citation is its whole token, so `<§>FS-loginUser` is no citation
/// of `FS-login` — `check` reports it dangling and `refs FS-login` does not list
/// it (§FS-citations.recognized.marker-start, §REQ-one-answer.identity.declarations).
pub fn citation_is_its_whole_token() -> Verdict {
    let tree = Tree::new(
        "r1-citation-prefix",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                "# FS-login: Login\n\nLogs in.\n\n# FS-other: Other\n\nSee \u{a7}FS-loginUser.\n",
            ),
        ],
    );
    let refs = stdout(&tree.grund(&["refs", "FS-login"]));
    let check = json_lines(&stdout(&tree.grund(&["check", "--format", "json"])));
    let dangling = check
        .iter()
        .any(|f| f["code"] == "dangling-citation" && f["line"] == 7);
    let listed = refs.contains("docs/fs.md:7");
    if listed || !dangling {
        return Err(format!(
            "refs FS-login lists docs/fs.md:7: {listed}; check reports \u{a7}FS-loginUser \
             dangling: {dangling}"
        ));
    }
    Ok(())
}

/// r2, row R01: a path argument narrows what is reported, never what resolves —
/// the ID `check sub` resolves, `grund <ID> sub` shows
/// (§REQ-one-answer.scope.narrowed, §REQ-one-answer.identity.declarations).
pub fn a_narrowed_run_resolves_what_check_resolves() -> Verdict {
    let tree = Tree::new(
        "r2-path-argument",
        &[
            ("grund.toml", CONFIG_NUMBER),
            ("docs/fs.md", "# FS-001-a: Alpha\n\nAlpha body.\n"),
            ("sub/notes.md", "Uses \u{a7}FS-001-a.\n"),
        ],
    );
    let check = tree.grund(&["check", "sub"]).status.success();
    let show = tree.grund(&["FS-001-a", "sub"]);
    if check != show.status.success() {
        return Err(format!(
            "check sub succeeds: {check}; grund FS-001-a sub succeeds: {} ({})",
            show.status.success(),
            stderr(&show).trim()
        ));
    }
    Ok(())
}

/// Row G06, closed: the subject `each FS` is refused for one reason as a rule's
/// subject and as a selector, which keeps its literal-subject reason in front and
/// appends the rule's after ` — ` (§FS-rule-refusals.selectors,
/// §REQ-one-answer.identity).
pub fn a_lowercase_subject_is_refused_once() -> Verdict {
    let config = format!("{CONFIG_SLUG}[[kinds]]\nkind = \"GOAL\"\nfile = \"docs/goals.md\"\n");
    let tree = Tree::new(
        "g06-lowercase-each",
        &[
            ("grund.toml", config.as_str()),
            ("docs/fs.md", "# FS-a: A\n\nServes \u{a7}GOAL-a.\n"),
            ("docs/goals.md", "# GOAL-a: A\n\nThe aim.\n"),
        ],
    );
    let selector = reason(&tree.error(&["list", "--selector", "each FS"]));
    agree(
        "the rule subject / the selector",
        reason(&tree.error(&["check", "--rule", "each FS must cite at least one GOAL."])),
        selector
            .rsplit_once(" — ")
            .map_or(selector.as_str(), |(_, appended)| appended)
            .to_string(),
    )
}

/// Row G09: the default homeless kind `code` is one verdict whether a `[citations]`
/// target or `list --kind` names it (§REQ-one-answer.identity.kinds).
pub fn the_homeless_kind_is_one_verdict() -> Verdict {
    let files = [("docs/fs.md", "# FS-a: A\n\nBody.\n")];
    let listed = Tree::new(
        "g09-list",
        &[&[("grund.toml", CONFIG_SLUG)][..], &files].concat(),
    );
    let table = format!("{CONFIG_SLUG}[citations.FS]\nmust = [\"code\"]\n");
    let tabled = Tree::new(
        "g09-table",
        &[&[("grund.toml", table.as_str())][..], &files].concat(),
    );
    agree(
        "the table's `code` is non-citable / list --kind's is",
        tabled.error(&["check"]).contains("non-citable"),
        listed
            .error(&["list", "--kind", "code"])
            .contains("non-citable"),
    )
}

/// Row R02: in a workspace, a citation the root `check` resolves is one `check
/// docs` resolves (§REQ-one-answer.scope.narrowed).
pub fn a_narrowed_workspace_run_resolves_the_alias() -> Verdict {
    let tree = Tree::new(
        "r02-workspace-path",
        &[
            ("grund.toml", WORKSPACE),
            ("docs/fs.md", CITES_MEMBER),
            ("api/grund.toml", MEMBER),
            ("api/docs/api.md", MEMBER_DOC),
        ],
    );
    let alias = |args: &[&str]| {
        let output = tree.grund(args);
        format!("{}{}", stdout(&output), stderr(&output)).contains("unknown project alias")
    };
    agree(
        "check reports `unknown project alias` / check docs does",
        alias(&["check"]),
        alias(&["check", "docs"]),
    )
}
