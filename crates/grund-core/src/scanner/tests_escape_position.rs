//! Test module: the escape position is not a citation (§FS-check.1.1.9) — the
//! carve-out the bare-token pass of §FS-check.1.1 reaches before its strict gate
//! (§AR-scanner.2.3.1).

use crate::config::Config;
use crate::grammar::in_escape_position;
use crate::testing::{numbered_config, scan_findings, test_root, write};

/// The line that carries the qualified escape, kept as a constant so the
/// predicate below is asked of the same bytes the scan reads.
const QUALIFIED_LINE: &str =
    "Across a namespace the illustration reads <§>api/FS-002-session here.";

fn escape_tree(name: &str) -> (std::path::PathBuf, Config) {
    let root = test_root(name);
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: User login\n\nCreates a session.\n",
    );
    write(
        &root.join("docs/guide.md"),
        &format!(
            concat!(
                "# Guide\n",
                "\n",
                "Login is specified in FS-001-login, an ordinary bare citation.\n",
                "\n",
                "The shape is written <§>FS-001-login in prose, and `<§>FS-001-login.3.1`\n",
                "inside an inline-code span. {}\n",
            ),
            QUALIFIED_LINE
        ),
    );
    // No apostrophe on any source line: §FS-check.1.1.3 opens a string-literal
    // state at an unescaped `'` and would defuse the rest of the line, which
    // would make this case pass for the wrong reason.
    write(
        &root.join("src/session.rs"),
        concat!(
            "//! Session handling, grounded in FS-001-login.\n",
            "\n",
            "/// Opens a session. The shape of a citation is written <§>FS-001-login here.\n",
            "pub fn open() {}\n",
            "\n",
            "// A line comment illustrating <§>FS-001-login too.\n",
        ),
    );
    let mut config = numbered_config(root.clone());
    config.strict = false;
    (root, config)
}

fn citation_sites(findings: &crate::model::Findings) -> Vec<(String, usize, bool)> {
    let mut sites = findings
        .citations
        .iter()
        .map(|cite| {
            (
                cite.file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("<none>")
                    .to_string(),
                cite.line,
                cite.has_marker,
            )
        })
        .collect::<Vec<_>>();
    sites.sort();
    sites
}

/// §FS-check.1.1.9: under `[reference] strict = false` an escaped ID yields no
/// citation in Markdown prose, in an inline-code span, in a `//` comment or in a
/// `///` doc-comment — the four contexts where a bare token is otherwise live —
/// while the ordinary bare citations beside them stay live. Each escaped site is
/// still recorded as an escaped citation, which is what keeps the
/// `escaped-citation-resolves` suggestion of §FS-check.2.3.1 the only report of
/// such a site.
#[test]
fn an_escape_yields_no_citation_in_prose_inline_code_or_a_comment() {
    let (root, config) =
        escape_tree("an_escape_yields_no_citation_in_prose_inline_code_or_a_comment");
    let findings = scan_findings(&config, &root);

    assert_eq!(
        citation_sites(&findings),
        vec![
            ("guide.md".to_string(), 3, false),
            ("session.rs".to_string(), 1, false),
        ],
        "only the two unescaped bare tokens are citations: {:?}",
        findings.citations
    );
    let escaped = findings
        .escaped_citations
        .iter()
        .map(|cite| cite.id.slug.clone().unwrap_or_default())
        .count();
    assert_eq!(
        escaped, 5,
        "every escaped site is still recorded, in prose, in inline code, across a \
         namespace, in a doc-comment and in a line comment: {:?}",
        findings.escaped_citations
    );
}

/// §FS-check.1.1.9: the rule is mode-independent. Turning `strict` back on drops
/// the two ordinary bare citations, which is the strict default, and changes
/// nothing about the escapes — they were never citations in either mode, and the
/// escaped record is identical.
#[test]
fn the_escape_holds_under_both_strict_modes() {
    let (root, mut config) = escape_tree("the_escape_holds_under_both_strict_modes");
    let lenient = scan_findings(&config, &root);
    config.strict = true;
    let strict = scan_findings(&config, &root);

    assert!(
        citation_sites(&strict).is_empty(),
        "under strict = true the bare tokens are text and the escapes are still \
         nothing: {:?}",
        strict.citations
    );
    assert_eq!(
        lenient.escaped_citations.len(),
        strict.escaped_citations.len(),
        "the escaped record does not depend on the mode"
    );
}

/// §FS-check.1.1.9: both escape spellings are exempt **by the escape**, not by
/// §FS-workspace.1.3's unmarked-`alias/ID` rule. No scan result can tell those two
/// reasons apart — the qualified form yields no citation either way — so the claim
/// is pinned where it is decidable: the escape predicate itself answers for the
/// qualified token's start column, and the scanner asks it before the namespace
/// guard and before the strict gate (§AR-scanner.2.3.1).
#[test]
fn the_escape_and_not_the_namespace_guard_exempts_the_qualified_form() {
    let unqualified = "The shape is written <§>FS-001-login in prose.";
    let unqualified_start = unqualified.find("FS-001-login").expect("the bare token");
    assert!(
        in_escape_position(unqualified, unqualified_start, "§"),
        "`<§>ID` is exempt by the escape"
    );

    let qualified_start = QUALIFIED_LINE.find("api/").expect("the qualified token");
    assert!(
        in_escape_position(QUALIFIED_LINE, qualified_start, "§"),
        "`<§>alias/ID` is exempt by the same escape, asked of the token's own start \
         column — the alias, not the ID behind the slash"
    );

    let (root, config) =
        escape_tree("the_escape_and_not_the_namespace_guard_exempts_the_qualified_form");
    let findings = scan_findings(&config, &root);
    assert!(
        findings
            .citations
            .iter()
            .all(|cite| cite.namespace.is_none()),
        "and the scan records no qualified citation for it: {:?}",
        findings.citations
    );
}
