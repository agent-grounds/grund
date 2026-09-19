//! Focused tests for deterministic ID slug derivation (§FS-id.3).

use super::id::slugify_title;

#[test]
fn slug_normalizes_unicode_and_separator_runs() {
    assert_eq!(
        slugify_title("---Café___log---in---", "[a-z0-9][a-z0-9-]*"),
        "cafe-log-in"
    );
}

#[test]
fn slug_truncates_at_the_last_word_boundary_before_sixty_characters() {
    let title = "one-two-three-four-five-six-seven-eight-nine-ten-eleven-twelve-thirteen";
    let slug = slugify_title(title, "[a-z0-9][a-z0-9-]*");
    assert_eq!(
        slug,
        "one-two-three-four-five-six-seven-eight-nine-ten-eleven"
    );
    assert!(slug.len() <= 60);
}

#[test]
fn slug_derivation_is_repeatable() {
    let title = "Déjà vu: deterministic IDs";
    assert_eq!(
        slugify_title(title, "[a-z0-9][a-z0-9-]*"),
        slugify_title(title, "[a-z0-9][a-z0-9-]*")
    );
}
