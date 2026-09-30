//! Test module: the citing site's own unit reaches the published `cover` and
//! `refs` records, and only the two commands that publish it pay for computing
//! it (§FS-cover.3.2, §FS-refs.3.2, §AR-scanner.2.4.2).
//!
//! These are the API seams rather than the CLI: the two e2e cases pin the bytes,
//! and a Rust embedder reads `CoverCitation` and `RefHit` without going near a
//! formatter, so the pair is pinned once on each side of that line.

use std::path::{Path, PathBuf};

use super::*;
use crate::model::TextOverlays;
use crate::resolver::{
    load_classifying_workspace_context, load_workspace_context,
    load_workspace_context_with_overlays,
};
use crate::testing::{test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\
    project_name = \"root\"\n\n\
    [id]\n\
    format = \"{kind}-{slug}\"\n\
    slug_pattern = \"[a-z][a-z0-9-]*\"\n\
    named_sections = true\n\n\
    [[kinds]]\n\
    kind = \"FS\"\n\
    folder = \"docs\"\n\n\
    [scan]\n\
    include = [\"docs\"]\n\
    extensions = [\"md\"]\n";

/// One file carrying all four answers, the same shape the two e2e cases run: a
/// site above every declaration, one in the body's lead above every section, one
/// under a named section, and one under a numbered section. It holds no
/// declaration-local shorthand on purpose — a file that does gets the
/// classification pass as a side effect (`has_local_candidates`), and a fixture
/// built out of one would answer correctly for a reason these cases are not
/// about.
/// Written with every line inside a quoted fragment, never continued so that a
/// heading starts a source line: a `# <ID>:` at the head of a line in this file
/// would be a declaration of this file's own (§FS-check.3.8).
const ALPHA: &str = concat!(
    "# Alpha\n",
    "\n",
    "A preamble above every declaration cites \u{a7}FS-beta.\n",
    "\n",
    "# FS-alpha: Alpha\n",
    "\n",
    "The lead of the body, above every section, cites \u{a7}FS-beta.\n",
    "\n",
    "## terms: Terms\n",
    "\n",
    "A named section cites \u{a7}FS-beta.\n",
    "\n",
    "## 1. Inputs\n",
    "\n",
    "A numbered section cites \u{a7}FS-beta.\n",
);

fn four_sites(name: &str) -> PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(&root.join("docs/FS-alpha.md"), ALPHA);
    write(
        &root.join("docs/FS-beta.md"),
        "# FS-beta: Beta\n\nBeta body.\n",
    );
    root
}

/// `<line>|<enclosing_declaration>|<enclosing_section>` per site, so a row names
/// which of the four cases it is and both halves of the pair at once.
fn pair(line: usize, declaration: Option<&str>, section: Option<&str>) -> String {
    format!(
        "{line}|{}|{}",
        declaration.unwrap_or("-"),
        section.unwrap_or("-")
    )
}

/// §FS-cover.3.2: `cover()` carries the citing site's enclosing declaration and
/// accepted section on every record — `null` above every declaration, `null` in
/// the lead above every section, the handle under a named section, and the
/// number under a numbered one.
#[test]
fn cover_carries_the_citing_sites_enclosing_unit() {
    let root = four_sites("cover_carries_the_citing_sites_enclosing_unit");
    let output = cover(CoverOpts {
        path: root.clone(),
        path_provided: true,
    })
    .expect("cover");
    let rows = output
        .entries
        .iter()
        .flat_map(|entry| &entry.citations)
        .map(|citation| {
            pair(
                citation.line,
                citation.enclosing_declaration.as_deref(),
                citation.enclosing_section.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            pair(3, None, None),
            pair(7, Some("FS-alpha"), None),
            pair(11, Some("FS-alpha"), Some("terms")),
            // §AR-scanner.2.4.4: numbered and named sections alike, which is the
            // one point the issue's worked example had wrong.
            pair(15, Some("FS-alpha"), Some("1")),
        ]
    );
}

/// §FS-refs.3.2: the same four answers through `refs()`, the other published
/// record. `kind_title` is metadata of the run rather than of a hit, so it is
/// not on `RefHit` and is not what this case is about.
#[test]
fn refs_carries_the_citing_sites_enclosing_unit() {
    let root = four_sites("refs_carries_the_citing_sites_enclosing_unit");
    let output = refs(RefsOpts {
        path: root.clone(),
        path_provided: true,
        id: "FS-beta".into(),
        section: None,
        descendants: false,
    })
    .expect("refs");
    let rows = output
        .hits
        .iter()
        .map(|hit| {
            pair(
                hit.line,
                hit.enclosing_declaration.as_deref(),
                hit.enclosing_section.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            pair(3, None, None),
            pair(7, Some("FS-alpha"), None),
            pair(11, Some("FS-alpha"), Some("terms")),
            pair(15, Some("FS-alpha"), Some("1")),
        ]
    );
}

/// §AR-scanner.2.4.2, §GOAL-fast-feedback: the queries that publish none of the
/// three citing-side fields still skip the post-pass. `list`, `fmt`,
/// `complete_ids`, `sizes` and `batch` enter through `load_workspace_context`
/// and `show` through `load_workspace_context_with_overlays(_, _, _, false)`, so
/// those two loaders are where the skip has to hold; `refs` has its own entry
/// precisely so that turning the pass on for it did not turn it on for six.
///
/// This is the case that stands between §GOAL-fast-feedback and a later
/// one-line "simplification" that makes every query pay.
#[test]
fn the_queries_that_publish_no_enclosing_unit_still_skip_classification() {
    let root = four_sites("the_queries_that_publish_no_enclosing_unit_still_skip_classification");
    let classified = |path: &Path, classifying: bool| {
        let context = if classifying {
            load_classifying_workspace_context(path, true)
        } else {
            load_workspace_context(path, true)
        }
        .expect("load context");
        context
            .projects
            .iter()
            .flat_map(|project| &project.findings.citations)
            .filter(|citation| citation.enclosing_declaration.is_some())
            .count()
    };
    assert_eq!(classified(&root, false), 0, "load_workspace_context");
    let overlaid = load_workspace_context_with_overlays(&root, true, &TextOverlays::new(), false)
        .expect("load context with overlays");
    assert_eq!(
        overlaid
            .projects
            .iter()
            .flat_map(|project| &project.findings.citations)
            .filter(|citation| citation.enclosing_declaration.is_some())
            .count(),
        0,
        "load_workspace_context_with_overlays(.., false) — show's entry"
    );
    // Three of the four sites sit in a declaration body, so the classifying
    // loader is the one that answers: the assertion above is a skip, not an
    // absence of anything to find.
    assert_eq!(
        classified(&root, true),
        3,
        "load_classifying_workspace_context — refs's entry"
    );
}

/// A workspace whose member numbers its IDs and whose root does not, so the
/// citing project's grammar and the target project's spell the same `Id`
/// differently. The member's file is the citing one.
fn two_grammars(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "project_name = \"root\"\n",
            "\n",
            "[id]\n",
            "format = \"{kind}-{slug}\"\n",
            "slug_pattern = \"[a-z][a-z0-9-]*\"\n",
            "\n",
            "[[kinds]]\n",
            "kind = \"FS\"\n",
            "folder = \"docs\"\n",
            "\n",
            "[scan]\n",
            "include = [\"docs\"]\n",
            "extensions = [\"md\"]\n",
            "\n",
            "[workspace]\n",
            "members = [\"packages/sub\"]\n",
        ),
    );
    write(
        &root.join("packages/sub/grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "project_name = \"sub\"\n",
            "\n",
            "[id]\n",
            "format = \"{kind}-{number}-{slug}\"\n",
            "slug_pattern = \"[a-z][a-z0-9-]*\"\n",
            "\n",
            "[[kinds]]\n",
            "kind = \"FS\"\n",
            "folder = \"docs\"\n",
            "\n",
            "[scan]\n",
            "include = [\"docs\"]\n",
            "extensions = [\"md\"]\n",
        ),
    );
    write(
        &root.join("docs/FS-root-thing.md"),
        concat!("# FS-root-thing: Root\n", "\n", "Root body.\n"),
    );
    write(
        &root.join("packages/sub/docs/FS-001-sub-thing.md"),
        concat!(
            "# FS-001-sub-thing: Sub\n",
            "\n",
            "Sub leans on \u{a7}root/FS-root-thing.\n",
        ),
    );
    root
}

/// §FS-workspace.8.2.4, §FS-workspace.8.6: `enclosing_declaration` renders under
/// the **citing** project's `[id]` config, because the declaration it names sits
/// in the citing file — unlike `id`, which renders under the target's. Here the
/// citing member numbers its IDs and the target root does not, so the two
/// answers differ by the number: the target's grammar would spell the same
/// declaration `FS-sub-thing`.
///
/// No e2e golden distinguishes the two — measured, not assumed — so this is the
/// only case that holds the citing side.
#[test]
fn enclosing_declaration_renders_under_the_citing_projects_grammar() {
    let root = two_grammars("enclosing_declaration_renders_under_the_citing_projects_grammar");
    let covered = cover(CoverOpts {
        path: root.clone(),
        path_provided: true,
    })
    .expect("cover");
    let citing = covered
        .entries
        .iter()
        .flat_map(|entry| &entry.citations)
        .map(|citation| {
            format!(
                "{}|{}",
                citation.id,
                citation.enclosing_declaration.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        citing,
        vec!["root/FS-root-thing|FS-001-sub-thing".to_string()],
        "cover: id under the target's grammar, enclosing_declaration under the citing project's"
    );

    let queried = refs(RefsOpts {
        path: root.clone(),
        path_provided: true,
        id: "root/FS-root-thing".into(),
        section: None,
        descendants: false,
    })
    .expect("refs");
    assert_eq!(
        queried
            .hits
            .iter()
            .map(|hit| format!(
                "{}|{}",
                hit.id,
                hit.enclosing_declaration.as_deref().unwrap_or("-")
            ))
            .collect::<Vec<_>>(),
        vec!["FS-root-thing|FS-001-sub-thing".to_string()],
        "refs: the same split between target and citing grammar"
    );
}
