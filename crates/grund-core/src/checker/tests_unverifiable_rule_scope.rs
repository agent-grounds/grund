//! Test module: the scope a rule object kind is resolved in, and the one
//! failure that is not an invalid rule (§FS-rules.4.1).
//!
//! The behaviour these units sit under is pinned black-box in
//! `crates/grund-cli/tests/rules_contract/workspace_scope.rs` and the
//! `…-pinned-rule-…` e2e cases. What is here is the seam each one rests on: the
//! classification bit, the authored sentence surviving into the row, and the
//! alias set `init` resolves against being the one `check` loads.

use super::chapter_rules::{configured_rule_sentences, declared_workspace_vocabulary, vocabulary};
use crate::config::{Config, ProjectRecords, load_config};
use crate::resolver::load_workspace_projects;
use crate::rules::RuleAnchor;
use crate::rules::sentence::{RuleVocabulary, parse_rule};
use crate::testing::{scan_tree, test_root, write};
use crate::workspace::declared_member_schemas;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const PINNED: &str = "The operations chapter of each SEG must cite at least one workshop/OP.";

/// A project that declares a rule kind, plus `extra` config text and one rule
/// sentence. `workshop` is written beside it whether or not the project claims
/// it as a member, so what varies between the cases below is the `[workspace]`
/// line and nothing else.
fn project(name: &str, extra: &str, sentence: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            concat!(
                "grund_config_version = 1\nproject_name = \"root\"\n\n",
                "[reference]\nstrict = true\n\n",
                "[id]\nformat = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\n",
                "{}\n",
                "[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n\n",
                "[[kinds]]\nkind = \"SEG\"\nfolder = \"docs/segments\"\nindex = false\n\n",
                "[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n\n",
                "[scan]\ninclude = [\"docs\"]\n",
            ),
            extra
        ),
    );
    write(
        &root.join("docs/goals/GOAL-build.md"),
        "# GOAL-build: Every segment names how it is built.\n\nA segment nobody can build is a drawing.\n",
    );
    write(
        &root.join("docs/segments/SEG-drive.md"),
        "# SEG-drive: The drive segment.\n\nThe drive turns the wheels for \u{a7}GOAL-build.\n\n## operations: Operations\n\nBuilt somehow.\n",
    );
    write(
        &root.join("docs/rules/RULE-ops.md"),
        &format!(
            "# RULE-ops: {sentence}\n\nThe operations chapter carries the build evidence for \u{a7}GOAL-build.\n"
        ),
    );
    workshop(&root.join("workshop"));
    root
}

/// The sibling project whose `OP` kind the pinned object names.
fn workshop(root: &Path) {
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"workshop\"\n\n",
            "[reference]\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[[kinds]]\nkind = \"OP\"\nfolder = \"docs/ops\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\n",
        ),
    );
    write(
        &root.join("docs/ops/OP-weld.md"),
        "# OP-weld: Welding.\n\nTwo parts, one seam.\n",
    );
}

/// The vocabulary `init` resolves `config`'s rules against (§FS-rules.4.1).
fn declared(config: &Config) -> RuleVocabulary {
    declared_workspace_vocabulary(
        config.schema(),
        config.frame(),
        &declared_member_schemas(config),
    )
}

/// Whether `sentence` resolves, and when it does not, whether it is
/// §FS-rules.4.1's unverifiable case (`Err(true)`) or an invalid rule
/// (`Err(false)`).
fn verdict(config: &Config, sentence: &str) -> Result<(), bool> {
    let vocab = declared(config);
    match parse_rule(
        sentence,
        "RULE-ops".into(),
        RuleAnchor {
            path: "docs/rules/RULE-ops.md".into(),
            line: 1,
            column: None,
        },
        &vocab,
    ) {
        Ok((_, Some(_))) => Err(true),
        Ok((_, None)) => Ok(()),
        Err(_) => Err(false),
    }
}

/// §FS-rules.4.1.1: all seven rows. The absence of every workspace namespace is
/// the whole test — where the vocabulary holds namespaces and the alias was
/// rejected anyway, the answer was no and the rule is invalid.
#[test]
fn only_a_scope_holding_no_namespace_makes_an_object_kind_unverifiable() {
    let with = load_config(&project(
        "rule-scope-with-workspace",
        "[workspace]\nmembers = [\"workshop\"]\n",
        PINNED,
    ))
    .expect("load workspace config");
    // No `[workspace]` line at all — the member-scoped run's effective config.
    let without = load_config(&project("rule-scope-without-workspace", "", PINNED))
        .expect("load standalone config");

    let each = "The operations chapter of each SEG must cite at least one";
    for (config, object, expected) in [
        (&without, "workshop/OP", Err(true)),
        (&with, "workshop/OP", Ok(())),
        (&with, "workshop/NOPE", Err(false)),
        (&with, "typo/OP", Err(false)),
        (&without, "*/OP", Err(true)),
        (&with, "*/OP", Ok(())),
        (&with, "*/NOPE", Err(false)),
        // `GOAL` is declared locally, so `*/GOAL` resolves in either scope
        // without any namespace being searched.
        (&without, "*/GOAL", Ok(())),
        (&with, "*/GOAL", Ok(())),
    ] {
        assert_eq!(
            verdict(config, &format!("{each} {object}.")),
            expected,
            "{object} with workspace_in_scope={}",
            declared(config).workspace_in_scope()
        );
    }
}

/// §FS-rules.9.1: the bullet is the authored sentence, so an unverifiable rule
/// earns its row unchanged — which is what makes a member-scoped write equal to
/// the bytes a run holding the whole workspace produces.
#[test]
fn an_unverifiable_rule_keeps_its_row_and_its_authored_sentence() {
    let root = project("rule-scope-row", "", PINNED);
    let config = load_config(&root).expect("load standalone config");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan");
    let vocab = declared(&config);
    let Ok(rules) = configured_rule_sentences(&findings, config.schema(), config.frame(), &vocab)
    else {
        panic!("an unverifiable rule is not an invalid one");
    };

    assert_eq!(
        rules.rows,
        vec![("RULE-ops".to_string(), PINNED.to_string())],
        "the row is the sentence exactly as authored"
    );
    let reported = rules
        .unverifiable
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(reported.len(), 1, "{reported:?}");
    let diagnostic = &rules.unverifiable[0];
    assert_eq!(diagnostic.code, "invalid-rule", "the code does not move");
    assert_eq!(diagnostic.line, Some(1), "reported at the rule's heading");
    assert!(
        diagnostic
            .message
            .contains("unknown project alias workshop"),
        "{}",
        diagnostic.message
    );

    // And a genuinely invalid rule beside it is still `Err`, which is what
    // withholds the write (§FS-rules.4.1.2).
    write(
        &root.join("docs/rules/RULE-broken.md"),
        "# RULE-broken: Segments ought to be nice.\n\nNot a rule at all, for \u{a7}GOAL-build.\n",
    );
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("rescan");
    assert!(
        configured_rule_sentences(&findings, config.schema(), config.frame(), &vocab).is_err(),
        "an invalid rule beside an unverifiable one is still a refusal"
    );
}

/// §FS-rules.4.1: `init` resolves against the same aliases `check` loads, for a
/// nested workspace too — because both come from `expand_workspace_tree`. A
/// second spelling of the tree is how `init` and `check` would come to disagree
/// about one sentence in one directory.
#[test]
fn the_declared_vocabulary_holds_the_aliases_the_workspace_load_yields() {
    let root = project(
        "rule-scope-nested",
        "[workspace]\nmembers = [\"workshop\", \"api\"]\n",
        PINNED,
    );
    // `api` is itself a workspace root, so the tree has a level below the
    // members the root lists.
    write(
        &root.join("api/grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"api\"\n\n",
            "[reference]\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[workspace]\nmembers = [\"core\"]\n\n",
            "[[kinds]]\nkind = \"REQ\"\nfolder = \"docs/requirements\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\n",
        ),
    );
    write(
        &root.join("api/docs/requirements/REQ-latency.md"),
        "# REQ-latency: Answers inside a second.\n\nSlower than that is a different product.\n",
    );
    workshop(&root.join("api/core"));
    write(
        &root.join("api/core/grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"core\"\n\n",
            "[reference]\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[[kinds]]\nkind = \"OP\"\nfolder = \"docs/ops\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\n",
        ),
    );

    let config = load_config(&root).expect("load workspace config");
    let vocab = declared(&config);
    let mut loaded = ProjectRecords::of(config.clone());
    let loaded = load_workspace_projects(&mut loaded)
        .expect("load the workspace the way `check` does")
        .into_iter()
        .map(|project| project.alias)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        vocab
            .target_namespaces
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        loaded,
        "one expansion, one alias set"
    );
    assert!(vocab.workspace_in_scope(), "the run holds this workspace");
    assert!(
        vocab.target_namespaces["api/core"].contains("OP"),
        "a nested member's kinds are in its own namespace: {:?}",
        vocab.target_namespaces
    );
}

/// §FS-rules.4.1.1: the workspace read is best-effort — an expansion that fails
/// raises nothing — but a config that declares `[workspace]` still holds its own
/// project's namespace, so the run is never one that no workspace is in scope
/// for. The reader is standing at the workspace root; sending them to check from
/// the workspace root would be the one direction that cannot help.
#[test]
fn a_declared_workspace_holds_its_own_namespace_even_when_the_members_do_not_expand() {
    let root = project(
        "rule-scope-broken-workspace",
        "[workspace]\nmembers = [\"workshop\", \"absent\"]\n",
        PINNED,
    );
    let config = load_config(&root).expect("load workspace config");
    let mut expanding = ProjectRecords::of(config.clone());
    assert!(
        load_workspace_projects(&mut expanding).is_err(),
        "the fixture's workspace really does fail to expand"
    );

    let vocab = declared(&config);
    assert!(
        vocab.workspace_in_scope(),
        "a declared workspace is in scope whatever became of its members"
    );
    assert_eq!(
        vocab.target_namespaces.keys().cloned().collect::<Vec<_>>(),
        vec!["root".to_string()],
        "and what it holds is its own namespace, the entry an empty member list yields"
    );
    assert_eq!(
        vocab.target_kinds,
        vocabulary(config.schema(), config.frame()).target_kinds,
        "the local kinds are unaffected"
    );
    assert_eq!(
        verdict(&config, PINNED),
        Err(false),
        "so the unreachable member's alias is an invalid rule, not an unverifiable one"
    );

    // And `members = []` reaches that same verdict, which is the disagreement
    // between two spellings of one broken tree that the fallback removes.
    let empty = load_config(&project(
        "rule-scope-empty-workspace",
        "[workspace]\nmembers = []\n",
        PINNED,
    ))
    .expect("load empty-workspace config");
    assert_eq!(verdict(&empty, PINNED), verdict(&config, PINNED));
}
