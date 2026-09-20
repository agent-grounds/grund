//! Test module: qualified-citation recognition (§AR-core-module-layout.1.5)

use super::scan_tree;
use crate::config::Config;
use crate::testing::{test_root, write};

/// §FS-workspace.1.3, §AR-workspace.3.1: a marker-prefixed qualified
/// citation (`<§>alias/<ID>`) is recognised; an unmarked `alias/<ID>` in
/// prose is text. There is one scan mode, not two.
#[test]
fn marked_qualified_citation_is_recognised_unmarked_one_is_text() {
    let root = test_root("marked_qualified_citation_is_recognised_unmarked_one_is_text");
    let body = format!(
        "# FS-login: Login\n\nMarked qualified: {marker}api/FS-login.\nBare path-shaped token: api/FS-login is just prose.\n",
        marker = "§"
    );
    write(&root.join("docs/functional-spec/FS-login.md"), &body);

    let mut config = Config::default_for(root.clone());
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.rebuild_grammar().expect("rebuild grammar");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");

    assert_eq!(findings.citations.len(), 1, "exactly one citation expected");
    let cite = &findings.citations[0];
    assert_eq!(cite.namespace.as_deref(), Some("api"));
    assert_eq!(cite.line, 3);
}

/// §AR-workspace.3.1: in non-strict mode, an unmarked `path/<ID>` must
/// not be silently promoted to a qualified citation. Was a regression on
/// the first workspace slice; this test pins the marker-anchored rule.
#[test]
fn non_strict_bare_token_with_slash_prefix_is_not_a_citation() {
    let root = test_root("non_strict_bare_token_with_slash_prefix_is_not_a_citation");
    write(
        &root.join("docs/functional-spec/FS-login.md"),
        "# FS-login: Login\n\nA bare path-looking token api/FS-other in prose.\n",
    );

    let mut config = Config::default_for(root.clone());
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.strict = false;
    config.rebuild_grammar().expect("rebuild grammar");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");

    assert!(
        findings.citations.is_empty(),
        "non-strict mode must not turn `path/FS-x` in prose into a citation"
    );
}
