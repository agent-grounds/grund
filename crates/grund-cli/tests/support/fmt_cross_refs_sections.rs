//! Which citations `fmt` links when the heading they name is fenced, past the body's end,
//! reached through a stub, or takes no anchor: grund.91 to grund.94's trees.

use super::*;

/// §FS-fmt.6.4.1: grund.91's tree, a citation of a section whose only heading is
/// inside a fence. `fmt` links nothing in either mode, and `check` still reports
/// the section missing, as both do on the same tree without the fenced line.
#[test]
fn a_fenced_section_heading_is_not_linked() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "fmt-cross-refs-fenced-section-skipped",
        "fmt-cross-refs-fenced-section-skipped-write",
        "check-fenced-section-heading-missing",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("fenced section heading", &outcomes);
}

/// §FS-fmt.6.4.1: grund.92's tree, a citation of a section whose only heading comes
/// after a plain heading has closed the declaration's body, in a scanned home and
/// through a stub to a target the walk does not reach. `fmt` links nothing, `check`
/// reports the section missing, and a section inside that target's body still links.
#[test]
fn a_section_heading_after_the_body_closes_is_not_linked() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "fmt-cross-refs-closed-body-section-skipped",
        "fmt-cross-refs-closed-body-section-skipped-write",
        "check-closed-body-section-heading-missing",
        "fmt-cross-refs-stub-closed-body-section-skipped",
        "fmt-cross-refs-stub-unscanned-target-section-linked",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("closed-body section heading", &outcomes);
}

/// §FS-fmt.6.2.1.1: grund.93's tree, a bare ID whose only record in the walk is a
/// stub to a Markdown target `[scan] include` leaves out. `fmt` links the target's
/// declaration heading, the anchor `show` reports, with one stub and with two, as it
/// does on the same tree with the target scanned.
#[test]
fn a_bare_id_through_a_stub_links_the_targets_heading() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "fmt-cross-refs-stub-unscanned-target-bare-id-linked",
        "fmt-cross-refs-two-stubs-unscanned-target-bare-id-linked",
        "fmt-cross-refs-stub-scanned-target-bare-id-linked",
        "show-stub-unscanned-target-bare-id-anchor-json",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("bare ID through a stub", &outcomes);
}

/// §FS-fmt.6.4.1: grund.94's trees, a citation of a section its declaration lacks
/// where the link would take no heading anchor: a source-file home behind a stub,
/// walked or not, and a Markdown home under the `none` profile. `fmt` links nothing,
/// and on the same homes the section the declaration has and the bare ID still link.
#[test]
fn a_missing_section_is_not_linked_where_the_link_takes_no_anchor() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "fmt-cross-refs-source-home-section-skipped",
        "fmt-cross-refs-source-home-section-skipped-write",
        "fmt-cross-refs-source-home-walked-section-skipped",
        "fmt-cross-refs-source-home-walked-section-skipped-write",
        "fmt-cross-refs-anchor-none-section-skipped",
        "fmt-cross-refs-anchor-none-section-skipped-write",
        "fmt-cross-refs-anchor-none-closed-body-section-skipped",
        "fmt-cross-refs-source-home-section-linked",
        "fmt-cross-refs-source-home-walked-section-linked",
        "fmt-cross-refs-anchor-none-section-linked",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("no-anchor missing section", &outcomes);
}
