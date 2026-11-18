//! §FS-values.8: port #1433's check → default fmt --write → check reproduction.
//! Refused forms retain their exact finding and citation occurrence; ordinary
//! citations, including another occurrence of the same target, still link.

#[path = "support/value_attempt_fixture.rs"]
mod support;
use support::{CONFIG, FORM, Fixture, SPLIT, VALUE};

macro_rules! refused_form {
    ($name:ident, $body:expr, $message:expr) => {
        #[test]
        fn $name() {
            let fixture = Fixture::new(stringify!($name));
            fixture.project(CONFIG, VALUE, $body);
            let expected = fixture.read("docs/offer.md");
            fixture.preserves_refusal("docs/offer.md", &expected, Some($message));
        }
    };
}

refused_form!(
    split,
    "The price is `9999`\n(§CONST-field-price.1) per field.",
    SPLIT
);
refused_form!(
    two_spaces,
    "The price is `9999`  (§CONST-field-price.1) per field.",
    FORM
);
refused_form!(
    tab,
    "The price is `9999`\t(§CONST-field-price.1) per field.",
    FORM
);
refused_form!(
    missing_space,
    "The price is `9999`(§CONST-field-price.1) per field.",
    FORM
);
refused_form!(
    absent_parentheses,
    "The price is `9999` §CONST-field-price.1 per field.",
    FORM
);
refused_form!(
    inner_whitespace,
    "The price is `9999` ( §CONST-field-price.1) per field.",
    FORM
);

#[test]
fn ordinary_occurrence_sharing_refused_target_still_links() {
    let fixture = Fixture::new("same-target");
    fixture.project(
        CONFIG,
        VALUE,
        "The price is `9999`  (§CONST-field-price.1); see §CONST-field-price.1.",
    );
    let expected = fixture.read("docs/offer.md").replace(
        "see §CONST-field-price.1.",
        "see [§CONST-field-price.1](../values/field-price.md#1-1200).",
    );
    fixture.preserves_refusal("docs/offer.md", &expected, Some(FORM));
}

#[test]
fn custom_marker_and_separator_preserve_attempt() {
    let fixture = Fixture::new("custom-syntax");
    fixture.project(
        &CONFIG.replace(
            "[schema.kinds.CONST]",
            "[schema]\nmarker = \"@\"\nsection_separator = \":\"\n[schema.kinds.CONST]",
        ),
        VALUE,
        "The price is `9999`  (@CONST-field-price:1).",
    );
    for path in ["docs/README.md", "values/README.md"] {
        fixture.write(path, &fixture.read(path).replace('§', "@"));
    }
    let expected = fixture.read("docs/offer.md");
    fixture.preserves_refusal("docs/offer.md", &expected, Some(FORM));
}

#[test]
fn marked_root_split_retains_refusal() {
    let fixture = Fixture::new("marked-root");
    fixture.project(
        CONFIG,
        VALUE,
        "## 1. Price <!-- grund:value -->\n\
        ### 1.1. 1200\n\n## 2. Use\n\nThe price is `9999`\n(§DOC-offer.1.1).",
    );
    let expected = fixture.read("docs/offer.md");
    fixture.preserves_refusal("docs/offer.md", &expected, Some(SPLIT));
}

#[test]
fn qualified_chapter_attempt_uses_member_authority() {
    let fixture = Fixture::new("workspace-chapter");
    fixture.write(
        "grund.toml",
        "grund_config_version = 1\n\
        [workspace]\nmembers = [\"power\", \"shop\"]\ninclude_root = false\n",
    );
    fixture.write(
        "power/grund.toml",
        "grund_config_version = 1\nproject_name = \"power\"\n\
        [id]\nnamed_sections = true\n\
        [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\nvalue_chapter = \"values\"\n\
        [scan]\ninclude = [\"docs\"]\n",
    );
    fixture.write(
        "shop/grund.toml",
        "grund_config_version = 1\nproject_name = \"shop\"\n\
        [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\n\
        [scan]\ninclude = [\"docs\"]\n",
    );
    fixture.write(
        "power/docs/price.md",
        "# AR-004-price: Price\n\n\
        ## values: Values\n### values.price: Price\n#### values.price.1: 1200\n",
    );
    fixture.write(
        "shop/docs/offer.md",
        "# Offer\n\nThe price is `9999`\n\
        (§power/AR-004-price.values.price.1).\n",
    );
    let expected = fixture.read("shop/docs/offer.md");
    fixture.preserves_refusal("shop/docs/offer.md", &expected, Some(SPLIT));
}

#[test]
fn valid_binding_and_correctly_formed_mismatch_keep_existing_behavior() {
    for (name, literal, code) in [
        ("valid", "1200", None),
        ("mismatch", "9999", Some("value-mismatch")),
    ] {
        let fixture = Fixture::new(name);
        fixture.project(
            CONFIG,
            VALUE,
            &format!("Price: `{literal}` (§CONST-field-price.1)."),
        );
        let before = fixture.check(code, None);
        let original = fixture.read("docs/offer.md");
        assert_eq!(fixture.run(&["fmt", "--check"]).0, 0);
        assert_eq!(fixture.run(&["fmt", "--write"]).0, 0);
        assert_eq!(fixture.read("docs/offer.md"), original);
        assert_eq!(fixture.check(code, None), before);
    }
}

#[test]
fn non_attempts_link_normally() {
    let fixture = Fixture::new("ordinary");
    fixture.project(
        CONFIG,
        VALUE,
        "Bare: §CONST-field-price.1.\n\n\
        Unbackticked: 9999 (§CONST-field-price.1).\n\n\
        Non-value target: `9999`  (§DOC-offer).",
    );
    fixture.check(None, None);
    let original = fixture.read("docs/offer.md");
    assert_eq!(fixture.run(&["fmt", "--check"]).0, 1);
    assert_eq!(fixture.run(&["fmt", "--write"]).0, 0);
    assert_eq!(
        fixture.read("docs/offer.md"),
        original
            .replace(
                "§CONST-field-price.1",
                "[§CONST-field-price.1](../values/field-price.md#1-1200)"
            )
            .replace("§DOC-offer)", "[§DOC-offer](offer.md#doc-offer-offer))")
    );
    fixture.check(None, None);
    assert_eq!(fixture.run(&["fmt", "--check"]).0, 0);
}

#[test]
fn existing_root_space_aim_refusal_stays_protected() {
    let fixture = Fixture::new("aim-refusal");
    fixture.project(
        CONFIG,
        &VALUE.replace("1200", "12 00"),
        "The price is `9999` (§CONST-field-price).",
    );
    let expected = fixture.read("docs/offer.md");
    fixture.preserves_refusal("docs/offer.md", &expected, None);
}
