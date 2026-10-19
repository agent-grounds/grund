//! The public identity record remains constructible and exhaustively matchable
//! until the scheduled construction change (§FS-distribution.3.1.2), while
//! presentation lives outside identity (§FS-config.3.2.8).

use grund_core::Id;

#[test]
fn public_id_keeps_its_three_field_construction_and_destructuring() {
    let id = Id {
        kind: "ADR".into(),
        num: Some(1),
        slug: None,
    };
    let Id { kind, num, slug } = id;
    assert_eq!(kind, "ADR");
    assert_eq!(num, Some(1));
    assert_eq!(slug, None);
}
