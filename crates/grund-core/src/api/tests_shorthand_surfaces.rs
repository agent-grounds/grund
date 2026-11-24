//! Test module: the number-only shorthand as the *consumer* surfaces see it —
//! ID rendering, `grund <ID>` and its ambiguity listing, the LSP snapshot, and
//! the terminal/editor client matchers (§FS-check.1.2,
//! §DF-number-only-citation-shorthand). Split out of `tests_shorthand.rs`,
//! which pins recognition, resolution, and the check error: these cases fail
//! when a surface stops accepting the shorthand, not when the scanner changes
//! its mind about one (§AR-core-module-layout.1).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use std::collections::BTreeMap;

use super::*;
use crate::grammar::{NAMED_COMPONENT_PATTERN, render_id};
use crate::model::Id;
use crate::queries::{LspSnapshotOpts, ShowFormat, ShowOpts, ShowQueryError};
use crate::scanner::resolve_id_arg;
use crate::testing::{canonical_test_path, numbered_config, scan_tree, test_root, write};
use crate::writers::{ITERM2_SNIPPET, KITTY_SNIPPET, VSCODE_EXTENSION_JS, WEZTERM_SNIPPET};
use regex::Regex;

// §AR-scanner.2.6.11: `render_id` reduces a partial `Id` by the same rule the
// shorthand pattern is derived from, so an unresolved shorthand prints as
// `FS-042` rather than leaking the raw `{slug}` placeholder into a report.
#[test]
fn render_id_prints_a_slugless_id_as_the_shorthand() {
    let root = test_root("render_id_prints_a_slugless_id_as_the_shorthand");
    let config = numbered_config(root.to_path_buf());
    let shorthand = Id {
        kind: "FS".into(),
        num: Some(42),
        slug: None,
    };
    assert_eq!(render_id(&config.grammar, &shorthand), "FS-042");
    assert_eq!(
        render_id(
            &config.grammar,
            &Id {
                kind: "FS".into(),
                num: Some(42),
                slug: Some("user-login".into()),
            }
        ),
        "FS-042-user-login"
    );
}

// §FS-check.1.2.6 (§FS-citations.shorthand.cli) / §FS-show.1.2: a query persists
// nothing, so the shorthand is simply expanded at the CLI boundary. This is also
// what makes a clicked `<§>FS-042` open (§FS-integrations.3.1).
#[test]
fn shorthand_resolves_as_a_query_argument() {
    let root = test_root("shorthand_resolves_as_a_query_argument");
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\nLead.\n",
    );
    let config = numbered_config(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan");

    let (id, section) =
        resolve_id_arg("FS-042", config.schema(), config.frame(), &findings).expect("resolve");
    assert_eq!(render_id(&config.grammar, &id), "FS-042-user-login");
    assert_eq!(section, None);

    let (id, section) =
        resolve_id_arg("FS-042.1", config.schema(), config.frame(), &findings).expect("resolve");
    assert_eq!(render_id(&config.grammar, &id), "FS-042-user-login");
    assert_eq!(section.as_deref(), Some("1"));

    // A full ID is unaffected, and an unknown shorthand keeps its written
    // form so the caller's own "not found" path names what was asked for.
    let (id, _) =
        resolve_id_arg("FS-999", config.schema(), config.frame(), &findings).expect("resolve");
    assert_eq!(render_id(&config.grammar, &id), "FS-999");
}

// §FS-show.2.2.1: an ambiguous shorthand argument is a query failure that
// lists every candidate rather than picking one.
#[test]
fn ambiguous_shorthand_argument_lists_every_candidate() {
    let root = test_root("ambiguous_shorthand_argument_lists_every_candidate");
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\nLead.\n",
    );
    write(
        &root.join("docs/functional-spec/FS-042-user-logout.md"),
        "# FS-042-user-logout: User logout\n\nLead.\n",
    );
    let config = numbered_config(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan");

    let err = resolve_id_arg("FS-042", config.schema(), config.frame(), &findings)
        .expect_err("ambiguous");
    assert_eq!(
        format!("{err:#}"),
        "ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)"
    );
}

/// §FS-show.2.2.1 / §FS-errors.5.2: the shorthand's candidates are IDs, not
/// `{path, line}` sites — listing either home under `sites` would tell a
/// JSON consumer to delete one of two *correct* declarations — so the
/// public `show` query never carries the typed sites carrier for this
/// refusal, unlike the two-homes and ambiguous-section refusals.
#[test]
fn ambiguous_shorthand_query_carries_no_sites() {
    let root = test_root("ambiguous_shorthand_query_carries_no_sites");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\nLead.\n",
    );
    write(
        &root.join("docs/functional-spec/FS-042-user-logout.md"),
        "# FS-042-user-logout: User logout\n\nLead.\n",
    );

    let Err(err) = show(
        "FS-042",
        ShowOpts {
            path: root.to_path_buf(),
            format: ShowFormat::Json,
            ..ShowOpts::default()
        },
    ) else {
        panic!("§FS-show.2.2.1: an ambiguous shorthand argument must refuse");
    };
    assert!(
        err.downcast_ref::<ShowQueryError>().is_none(),
        "the shorthand refusal must stay a plain message, not the sites carrier"
    );
}

/// §FS-lsp.1.4: a shorthand already in the document navigates like any other
/// citation. The snapshot carries the canonical target while the range stays
/// the written token, which is what makes hover, go-to-definition,
/// references, document links, and highlight all work without any of them
/// knowing the shorthand exists.
#[test]
fn lsp_snapshot_navigates_a_shorthand_citation() {
    let root = test_root("lsp_snapshot_navigates_a_shorthand_citation");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{number}-{slug}\"\n",
    );
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\nLead.\n\n## 1. Inputs\n\nStuff.\n",
    );
    write(&root.join("src/lib.rs"), "//! \u{a7}FS-042.1\n");
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.clone(),
        path_provided: true,
        open_documents: BTreeMap::new(),
    })
    .expect("lsp snapshot");

    let citation = snapshot
        .citations
        .iter()
        .find(|citation| citation.display_path == "src/lib.rs")
        .expect("shorthand citation");
    assert_eq!(
        citation.text, "\u{a7}FS-042.1",
        "range covers what was typed"
    );
    assert_eq!(citation.query_id, "FS-042-user-login.1");
    assert_eq!(citation.declaration_query_id, "FS-042-user-login");
    assert_eq!(
        citation.target_path.as_deref().map(canonical_test_path),
        Some(canonical_test_path(
            &root.join("docs/functional-spec/FS-042-user-login.md")
        ))
    );
    assert_eq!(citation.target_line, Some(5), "jumps to the cited section");

    // The §FS-check.3.13 finding reaches the editor as a diagnostic.
    assert!(
        snapshot
            .report
            .errors
            .iter()
            .any(|finding| finding.code == "shorthand-citation"),
        "{:?}",
        snapshot.report.errors
    );
}

/// §FS-integrations.3.1: the clients need no shorthand matcher of their own —
/// the shared citation shape already accepts one, in every form. This pins
/// that, because the spec claims it and six hand-written regexes would
/// otherwise be free to drift from the engine.
#[test]
fn client_matchers_already_accept_the_shorthand() {
    let citation_shape = Regex::new(&client_citation_shape()).expect("client citation shape");
    for (text, expected) in [
        ("see \u{a7}FS-042 here", "\u{a7}FS-042"),
        ("see \u{a7}FS-042.1 here", "\u{a7}FS-042.1"),
        ("see \u{a7}api/FS-042 here", "\u{a7}api/FS-042"),
        (
            "see \u{a7}FS-042-user-login here",
            "\u{a7}FS-042-user-login",
        ),
    ] {
        assert_eq!(
            citation_shape.find(text).map(|found| found.as_str()),
            Some(expected),
            "client matcher must claim {text:?}"
        );
    }
    // The same shape is what every client artifact embeds.
    for artifact in [
        WEZTERM_SNIPPET,
        KITTY_SNIPPET,
        ITERM2_SNIPPET,
        VSCODE_EXTENSION_JS,
    ] {
        assert!(
            artifact.contains("[A-Z][A-Z0-9]*-[a-z0-9][a-z0-9-]*"),
            "client artifact lost the shared citation shape"
        );
    }
}

/// The citation shape every client matcher embeds: the ID shape of
/// §FS-integrations.3.1, then a suffix whose components are digits or the
/// named handle of §FS-config.3.3.1. The named alternative is read from the
/// engine's own pattern, so the clients cannot drift from the section grammar.
fn client_citation_shape() -> String {
    format!(
        r"[^\w\s]{{1,3}}(?:[a-z][a-z0-9-]*/)?[A-Z][A-Z0-9]*-[a-z0-9][a-z0-9-]*(?:\.(?:[0-9]+|{NAMED_COMPONENT_PATTERN}))*"
    )
}

/// Every citation matcher an artifact carries, unescaped from its host syntax
/// (a Lua string doubles `\`, a JavaScript literal escapes `/`).
fn embedded_citation_matchers(artifact: &str) -> Vec<String> {
    let mut matchers = Vec::new();
    for start in [r"[^\w\s]{1,3}", r"[^\\w\\s]{1,3}"] {
        for (at, _) in artifact.match_indices(start) {
            let rest = &artifact[at..];
            let end = ["'", "\n", "/g;"]
                .iter()
                .filter_map(|terminator| rest.find(terminator))
                .min()
                .unwrap_or(rest.len());
            matchers.push(
                rest[..end]
                    .trim_end()
                    .replace(r"\\", r"\")
                    .replace(r"\/", "/"),
            );
        }
    }
    // Prose that names the marker class is not a matcher; one carries the ID shape.
    matchers.retain(|matcher| matcher.contains("[A-Z][A-Z0-9]*-[a-z0-9][a-z0-9-]*"));
    matchers
}

/// §FS-integrations.3.1: the matchers keep the whole `.<section>` suffix,
/// named and mixed paths included, so a click opens the cited section rather
/// than the declaration heading (§FS-integrations.3.1, §FS-integrations.3.2).
#[test]
fn client_matchers_keep_named_and_mixed_section_paths() {
    let text = "see \u{a7}FS-login.requirements.lockout, \u{a7}FS-login.1.goals and \
                \u{a7}FS-login.goals.3 here; \u{a7}FS-login.1.2 too, and \u{a7}FS-login.";
    let expected = [
        "\u{a7}FS-login.requirements.lockout",
        "\u{a7}FS-login.1.goals",
        "\u{a7}FS-login.goals.3",
        "\u{a7}FS-login.1.2",
        "\u{a7}FS-login",
    ];
    let shared = Regex::new(&client_citation_shape()).expect("client citation shape");
    let found: Vec<&str> = shared.find_iter(text).map(|m| m.as_str()).collect();
    assert_eq!(
        found, expected,
        "the shared shape must keep every section path whole"
    );
    for (name, artifact, count) in [
        ("wezterm", WEZTERM_SNIPPET, 1),
        ("kitty", KITTY_SNIPPET, 2),
        ("iterm2", ITERM2_SNIPPET, 1),
        ("vscode", VSCODE_EXTENSION_JS, 1),
    ] {
        let matchers = embedded_citation_matchers(artifact);
        assert_eq!(
            matchers.len(),
            count,
            "{name}: citation matchers found {matchers:?}"
        );
        for matcher in matchers {
            let regex = Regex::new(&matcher).expect("client matcher compiles");
            let found: Vec<&str> = regex.find_iter(text).map(|m| m.as_str()).collect();
            assert_eq!(
                found, expected,
                "{name} matcher {matcher:?} must keep named and mixed section paths whole"
            );
        }
    }
}
