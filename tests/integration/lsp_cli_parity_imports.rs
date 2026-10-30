//! The import fixture of the parity sweep (§FS-imports.editor, §FS-lsp.4): over one
//! consumer that imports a sibling tree, every navigation answer the editor gives
//! through the import is the command line's answer over the same tree, and the
//! diagnostics for a section citation, a missing section and a value binding through
//! it are `check`'s findings.

use crate::lsp_import_session::{Composition, DEPENDENCY_CONFIG, Session, consumer_config};
use crate::{cli_findings, lsp_findings};
use serde_json::{Value, json};
use std::path::PathBuf;

const CITING: &str = "enterprise/src/Example.java";
/// A whole ID at UTF-16 column 26 of line 0; a section at 11 and a missing one at 53
/// of line 1; a value bound right at 17 and wrong at 51 of line 2.
const CITATIONS: &str = "// Registers providers as \u{a7}native-image/FS-security-providers requires.\n\
    // Section \u{a7}native-image/FS-security-providers.2 and \u{a7}native-image/FS-security-providers.9.\n\
    // Costs `1200` (\u{a7}native-image/CONST-price.1), not `1250` (\u{a7}native-image/CONST-price.1).\n\
    class Example {}\n";
const WHOLE: &str = "native-image/FS-security-providers";

fn composition(name: &str) -> Composition {
    let composition = Composition::new(name);
    composition.write(CITING, CITATIONS);
    composition
}

/// What `grund <args>` prints in the consumer, `None` where it refuses.
fn cli(composition: &Composition, args: &[&str]) -> Option<String> {
    let (code, stdout) = composition.grund("enterprise", args);
    (code == Some(0)).then_some(stdout)
}

/// A path the command line spelled from the consumer, or a `file:` URI, made canonical.
fn located(composition: &Composition, spelled: &str) -> PathBuf {
    let path = match url::Url::parse(spelled) {
        Ok(uri) if uri.scheme() == "file" => uri.to_file_path().expect("file uri"),
        _ => composition.path("enterprise").join(spelled),
    };
    path.canonicalize().unwrap_or(path)
}

/// Where `grund show <id>` finds `id`: its file and 1-based line.
fn shown_at(composition: &Composition, id: &str) -> Option<(PathBuf, u64)> {
    let shown = cli(composition, &["show", id, "--format", "json"])?;
    let shown: Value = serde_json::from_str(&shown).expect("show json");
    let path = located(composition, shown["path"].as_str()?);
    Some((path, shown["line"].as_u64()?))
}

fn completion(session: &mut Session, composition: &Composition, typed: &str) -> Vec<String> {
    session.open(CITING, &format!("// see {typed}\nclass Example {{}}\n"));
    let at = json!({"line": 0, "character": "// see ".len() + typed.chars().count()});
    let items = session.request(
        "textDocument/completion",
        json!({"textDocument": {"uri": composition.uri(CITING)}, "position": at}),
    );
    let mut labels = items["items"].as_array().cloned().unwrap_or_default();
    labels.sort_by_key(|item| item["label"].to_string());
    labels
        .iter()
        .map(|item| item["label"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// §FS-imports.editor: hover, definition, references, document links and completion
/// through the import are what `show`, `refs` and `complete ids` answer.
#[test]
fn navigation_through_an_import_is_the_cli_answer() {
    let composition = composition("parity-navigation");
    let mut session = Session::start(&composition, &["enterprise"], true);
    session.open(CITING, CITATIONS);
    let brief = cli(&composition, &["show", WHOLE, "--brief"]).expect("brief");
    let at = |line: u32, character: u32| {
        json!({"textDocument": {"uri": composition.uri(CITING)},
            "position": {"line": line, "character": character}})
    };

    let toc = cli(&composition, &["show", WHOLE, "--toc"]).expect("toc");
    assert_eq!(
        session.hover(CITING, 0, 32),
        Some(format!("{brief}\n{toc}"))
    );
    let section = cli(&composition, &["show", &format!("{WHOLE}.2")]).expect("section");
    assert_eq!(
        session.hover(CITING, 1, 20),
        Some(format!("{brief}{section}"))
    );
    assert_eq!(cli(&composition, &["show", &format!("{WHOLE}.9")]), None);
    assert_eq!(
        session.hover(CITING, 1, 60),
        None,
        "a missing section has no body"
    );
    let value = cli(&composition, &["show", "native-image/CONST-price.1"]).expect("value");
    assert_eq!(
        session.hover(CITING, 2, 22).as_deref(),
        Some(value.trim_end())
    );

    let definition = session.request("textDocument/definition", at(0, 32));
    let target = &definition[0];
    let defined = (
        located(&composition, target["targetUri"].as_str().expect("target")),
        target["targetRange"]["start"]["line"]
            .as_u64()
            .expect("line")
            + 1,
    );
    assert_eq!(Some(defined), shown_at(&composition, WHOLE));

    let mut references = session
        .request("textDocument/references", {
            let mut params = at(0, 32);
            params["context"] = json!({"includeDeclaration": false});
            params
        })
        .as_array()
        .expect("references")
        .iter()
        .map(|site| {
            let path = located(&composition, site["uri"].as_str().expect("uri"));
            (
                path,
                site["range"]["start"]["line"].as_u64().expect("line") + 1,
            )
        })
        .collect::<Vec<_>>();
    let refs = cli(&composition, &["refs", WHOLE, "--format", "json"]).expect("refs");
    let mut cited = refs
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("refs json"))
        .map(|site| {
            let path = located(&composition, site["path"].as_str().expect("path"));
            (path, site["line"].as_u64().expect("line"))
        })
        .collect::<Vec<_>>();
    references.sort();
    cited.sort();
    assert_eq!(references, cited);

    let links = session.request(
        "textDocument/documentLink",
        json!({"textDocument": {"uri": composition.uri(CITING)}}),
    );
    let links = links.as_array().expect("links");
    assert_eq!(links.len(), 5, "{links:?}");
    for link in links {
        let id = link["tooltip"]
            .as_str()
            .and_then(|tip| tip.strip_prefix("Open "))
            .expect("id");
        // A missing section opens its declaration, where `show` finds the ID.
        let shown = shown_at(&composition, id).or_else(|| shown_at(&composition, WHOLE));
        let (target, line) = link["target"]
            .as_str()
            .expect("target")
            .rsplit_once("#L")
            .expect("line");
        assert_eq!(
            Some((located(&composition, target), line.parse().expect("n"))),
            shown,
            "{id}"
        );
    }

    for typed in ["\u{a7}nat", "\u{a7}native-image/"] {
        let prefix = typed.trim_start_matches('\u{a7}');
        let offered = cli(&composition, &["complete", "ids", "--prefix", prefix]).expect("ids");
        let offered = offered.lines().map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            completion(&mut session, &composition, typed),
            offered,
            "{typed}"
        );
    }
    session.shutdown();
}

/// §FS-imports.editor: completion offers the aliases an import reached from the same alias
/// list and by the same rule as `complete ids`. At the import's `/` that is its nested
/// member's alias beside its IDs, and every alias the editor offers the command line
/// offers too. A plain member alias outside the import is not offered.
#[test]
fn completion_offers_the_aliases_beneath_an_import_as_complete_ids_does() {
    let composition = composition("parity-nested");
    let nested = format!("{DEPENDENCY_CONFIG}\n[workspace]\nmembers = [\"org\"]\n");
    composition.write("graal/substratevm/grund.toml", &nested);
    composition.write(
        "graal/substratevm/org/grund.toml",
        "grund_config_version = 2\nproject_name = \"org\"\n\n[schema.kinds.GOAL]\nfolders = [\"docs\"]\n",
    );
    composition.write(
        "graal/substratevm/org/docs/GOAL-reach.md",
        "# GOAL-reach: Reach\n",
    );
    let member = format!(
        "{}\n[workspace]\nmembers = [\"sub\"]\n",
        consumer_config("enterprise")
    );
    composition.write("enterprise/grund.toml", &member);
    composition.write(
        "enterprise/sub/grund.toml",
        "grund_config_version = 2\nproject_name = \"sub\"\n",
    );
    let mut session = Session::start(&composition, &["enterprise"], true);
    session.open(CITING, CITATIONS);
    let offered = |prefix: &str| {
        let offered = cli(&composition, &["complete", "ids", "--prefix", prefix]).expect("ids");
        offered.lines().map(str::to_string).collect::<Vec<_>>()
    };

    for typed in ["\u{a7}native-image/", "\u{a7}native-image/o"] {
        let editor = completion(&mut session, &composition, typed);
        assert!(
            editor.iter().any(|label| label == "native-image/org/"),
            "{typed}: {editor:?}"
        );
        assert_eq!(
            editor,
            offered(typed.trim_start_matches('\u{a7}')),
            "{typed}"
        );
    }
    assert!(
        completion(&mut session, &composition, "\u{a7}native-image/")
            .iter()
            .any(|label| label == WHOLE),
        "the import's IDs beside its nested alias"
    );
    for typed in ["\u{a7}n", "\u{a7}s", "\u{a7}"] {
        let cli = offered(typed.trim_start_matches('\u{a7}'));
        let editor = completion(&mut session, &composition, typed);
        let aliases = editor.iter().filter(|label| label.ends_with('/'));
        let unoffered = aliases
            .filter(|label| !cli.contains(label))
            .collect::<Vec<_>>();
        assert!(
            unoffered.is_empty(),
            "{typed}: {unoffered:?} beyond {cli:?}"
        );
        let reached = ["native-image/", "native-image/org/"];
        if typed != "\u{a7}s" {
            assert!(
                reached
                    .iter()
                    .all(|alias| editor.contains(&alias.to_string())),
                "{typed}: {editor:?}"
            );
        }
        assert!(!editor.contains(&"sub/".to_string()), "{typed}: {editor:?}");
    }
    session.shutdown();
}

/// §FS-imports.editor: a section citation, a missing section and a value binding through
/// an import are reported alike, the missing section and the wrong value on both.
#[test]
fn section_and_value_citations_through_an_import_agree() {
    let composition = composition("parity-findings");
    let consumer = composition.path("enterprise");
    let (cli, _) = cli_findings(&crate::binaries::grund(), &consumer).expect("check runs");
    let (lsp, _) = lsp_findings(&consumer);
    assert_eq!(cli, lsp);
    let codes = cli
        .iter()
        .map(|finding| (finding.line, finding.code.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            (4, "import-live"),
            (2, "missing-section"),
            (3, "value-mismatch")
        ]
    );
}
