//! grund#641's cases, run alone: a heading inside a fence in a doc-comment block is no
//! section on any surface (§FS-languages.fence.comment-block). Every case here also runs in
//! the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// §FS-languages.fence.comment-block: grund#641's tree, a heading inside a fence in a
/// `///`, `/** */` or docstring block is no section on any surface, while a citation in
/// the fence stays live, a value marker on a fenced heading is an illustration, and an
/// unclosed fence ends with its own comment block.
#[test]
fn a_fenced_heading_in_a_doc_comment_is_not_a_section() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "toc",
        "block-toc",
        "docstring-toc",
        "inner-tilde-toc",
        "go-toc",
        "full",
        "refs-json",
        "check-missing",
        "check-level",
        "check-duplicate",
        "fmt",
        "citation-live",
        "unclosed-ends-with-block",
        "value-marker",
    ]
    .iter()
    .map(|name| {
        let case = cases.join(format!("doc-comment-fenced-heading-{name}"));
        run_case(&root, &case, E2e)
    })
    .collect::<Vec<_>>();
    assert_every_case_passed("doc-comment fenced heading", &outcomes);
}
