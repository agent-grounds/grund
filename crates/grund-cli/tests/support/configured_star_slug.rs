// §FS-declarations.line.configured-slug: port grund.68's reader matrix.
use super::*;

#[test]
fn configured_star_slug_cli_contract() {
    let root = repo_root();
    let outcomes = [
        "configured-star-slug-list",
        "configured-star-slug-cover",
        "configured-star-slug-show-star",
        "configured-star-slug-bare-star",
        "configured-star-slug-refs-star",
        "configured-star-slug-uncited-refs-star",
        "configured-star-slug-show-comment-star",
        "configured-star-slug-show-docstring-star",
        "configured-star-slug-show-tail",
        "configured-star-slug-bare-tail",
        "configured-star-slug-refs-tail",
        "configured-star-slug-uncited-refs-tail",
        "configured-star-slug-show-comment-tail",
        "configured-star-slug-show-docstring-tail",
        "configured-star-slug-check",
    ]
    .iter()
    .map(|name| run_case(&root, &root.join("tests/e2e/cases").join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("configured star slug", &outcomes);
}
