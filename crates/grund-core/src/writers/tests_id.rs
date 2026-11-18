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

/// §FS-id.3 step 1: a combining mark inside a word is stripped, never read as a separator.
#[test]
fn slug_strips_a_combining_mark_inside_a_word() {
    for (title, slug) in [
        ("über", "uber"),
        ("Müller", "muller"),
        ("résumé", "resume"),
        ("naïve", "naive"),
        ("Ärger", "arger"),
        ("İstanbul", "istanbul"),
        ("Müller café", "muller-cafe"),
    ] {
        assert_eq!(slugify_title(title, "[a-z0-9][a-z0-9-]*"), slug, "{title}");
    }
}

/// §FS-id.3 step 6: a run of more than 60 characters with no `-` is cut at 60.
#[test]
fn slug_is_cut_at_sixty_characters_where_none_holds_a_hyphen() {
    let title = format!("{} word", "a".repeat(70));
    assert_eq!(slugify_title(&title, "[a-z0-9][a-z0-9-]*"), "a".repeat(60));
}

/// §FS-id.3 step 6: the cut is before the last `-` among the first 60, so a word that ends
/// at character 60 goes with the `-` after it.
#[test]
fn slug_is_cut_before_the_last_hyphen_among_its_first_sixty() {
    let title = format!("{} bbbb cc", "a".repeat(55));
    assert_eq!(slugify_title(&title, "[a-z0-9][a-z0-9-]*"), "a".repeat(55));
}
