//! CLI contracts for declaration-local numeric citations across check, refs, and
//! cover text/JSON surfaces (§FS-check.3.24, §FS-refs.2, §FS-cover.2), with
//! deterministic finding order (§FS-errors.4.1), and the remedy each finding names
//! clearing its site when it is followed (§FS-check.3.24.3).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("grund-cli-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).expect("create fixture");
    fs::write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"local-section-cli\"\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\n",
            "[output]\nrelative_paths = true\n\n",
            "[fmt.cross_refs]\nenabled = false\n",
        ),
    )
    .expect("write config");
    fs::write(
        root.join("docs/FS-a.md"),
        concat!(
            "# FS-a: A thing\n\n",
            "Valid \u{a7}2.\n",
            "Nested \u{a7}2.1.\n",
            "Missing \u{a7}9.9.\n",
            "Unsupported \u{a7}2.goals.\n",
            "Glued \u{a7}2abc.\n",
            "Malformed \u{a7}2..1 and \u{a7}2... and \u{a7}2..goals.\n",
            "Full valid \u{a7}FS-a.2.\n",
            "Full missing \u{a7}FS-a.9.\n\n",
            "## 2. Target\n\n",
            "### 2.1 Child\n",
        ),
    )
    .expect("write declaration");
    fs::write(
        root.join("docs/outside.md"),
        "# Notes\n\nOwnerless \u{a7}2.\n",
    )
    .expect("write ownerless file");
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .arg(root)
        .output()
        .expect("run grund")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

#[test]
fn check_text_reports_owned_missing_unsupported_and_ownerless_local_forms() {
    let root = fixture("local-section-check-text");
    let output = run(&root, &["check"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        concat!(
            "docs/FS-a.md:3: error: local section citation \u{a7}2; write \u{a7}FS-a.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`\n",
            "docs/FS-a.md:4: error: local section citation \u{a7}2.1; write \u{a7}FS-a.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`\n",
            // `FS-a` has no section 9.9, so the finding says so and offers the
            // escape (§FS-check.3.24.3); §FS-fmt.2.4.6 refuses the site, so
            // §FS-check.3.24.1 withholds the command clause.
            "docs/FS-a.md:5: error: local section citation \u{a7}9.9; FS-a has no section 9.9, so write a full citation or <§>9.9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:5: error: missing section FS-a.9.9\n",
            "docs/FS-a.md:6: error: unsupported local section citation \u{a7}2.goals; write a full citation or <§>2.goals to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:7: error: unsupported local section citation \u{a7}2abc; write a full citation or <§>2abc to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:8: error: unsupported local section citation \u{a7}2...; write a full citation or <§>2... to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:8: error: unsupported local section citation \u{a7}2..1; write a full citation or <§>2..1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:8: error: unsupported local section citation \u{a7}2..goals; write a full citation or <§>2..goals to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
            "docs/FS-a.md:10: error: missing section FS-a.9\n",
            "docs/outside.md:3: error: local section citation \u{a7}2 has no enclosing declaration; write a full citation or <§>2 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0\n",
        )
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn check_json_reports_the_same_local_verdicts() {
    let root = fixture("local-section-check-json");
    let output = run(&root, &["check", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let rows = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON finding"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 11, "{rows:?}");
    assert_eq!(rows[0]["code"], "local-section-citation");
    assert_eq!(
        rows[0]["message"],
        "local section citation \u{a7}2; write \u{a7}FS-a.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`"
    );
    assert_eq!(rows[2]["code"], "local-section-citation");
    assert_eq!(rows[3]["code"], "missing-section");
    assert_eq!(rows[9]["message"], "missing section FS-a.9");
    assert_eq!(rows[10]["path"], "docs/outside.md");
    assert_eq!(rows[10]["code"], "local-section-citation");
    assert_eq!(stderr(&output), "");
}

#[test]
fn refs_text_includes_local_spelling_in_whole_id_and_exact_section_queries() {
    let root = fixture("local-section-refs-text");
    let whole = run(&root, &["refs", "FS-a"]);
    assert_eq!(whole.status.code(), Some(0));
    assert_eq!(
        stdout(&whole),
        concat!(
            "docs/FS-a.md:3: \u{a7}2\n",
            "docs/FS-a.md:4: \u{a7}2.1\n",
            "docs/FS-a.md:5: \u{a7}9.9\n",
            "docs/FS-a.md:9: \u{a7}FS-a.2\n",
            "docs/FS-a.md:10: \u{a7}FS-a.9\n",
        )
    );
    let exact = run(&root, &["refs", "FS-a.2"]);
    assert_eq!(
        stdout(&exact),
        "docs/FS-a.md:3: \u{a7}2\ndocs/FS-a.md:9: \u{a7}FS-a.2\n"
    );
}

#[test]
fn refs_json_retains_local_text_but_returns_the_canonical_target() {
    let root = fixture("local-section-refs-json");
    let output = run(&root, &["refs", "FS-a", "--format", "json"]);
    let rows = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON ref"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 5, "{rows:?}");
    assert_eq!(rows[0]["id"], "FS-a");
    assert_eq!(rows[0]["section"], "2");
    assert_eq!(rows[0]["text"], "\u{a7}2");
    assert_eq!(rows[1]["section"], "2.1");
    assert_eq!(rows[2]["section"], "9.9");
}

#[test]
fn cover_json_counts_owned_local_edges_and_no_ownerless_guess() {
    let root = fixture("local-section-cover-json");
    let output = run(&root, &["cover", "--format", "json"]);
    let rows = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON cover row"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0]["citations"].as_array().unwrap().len(), 5);
    assert_eq!(rows[0]["citations"][0]["text"], "\u{a7}2");
    assert_eq!(rows[1]["path"], "docs/outside.md");
    assert_eq!(rows[1]["citations"], serde_json::json!([]));
}

/// What the finding tells the author to write at its site: the first word after
/// its first `write ` that carries the marker, a full citation or an escape alike.
/// Read from the message, never assumed, so the test below follows whatever
/// advice the binary actually gives.
fn the_written_remedy(message: &str) -> String {
    let (_, advice) = message
        .split_once("write ")
        .unwrap_or_else(|| panic!("the finding names what to write: {message:?}"));
    advice
        .split_whitespace()
        .find(|word| word.contains('\u{a7}'))
        .unwrap_or_else(|| panic!("the advice names a token to write: {message:?}"))
        .to_string()
}

/// §FS-check.3.24.3, followed the way an agent follows it: every
/// `local-section-citation` finding's own remedy is written over its token, and
/// the tree is then clean. The fixture is agent-grounds/grund#515's shape — prose
/// naming sections of other files inside a declaration whose own sections stop
/// at 2 — with a resolving site beside them, whose full citation clears it too.
/// Where the finding proposes the owner's full citation of a section the owner
/// lacks, following it only trades the error for `missing section`.
#[test]
fn following_each_local_section_findings_remedy_clears_its_site() {
    let root = std::env::temp_dir().join(format!(
        "grund-cli-local-section-remedy-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).expect("create fixture");
    fs::write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"local-section-remedy\"\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\n",
            "[output]\nrelative_paths = true\n\n",
            "[fmt.cross_refs]\nenabled = false\n",
        ),
    )
    .expect("write config");
    let file = root.join("docs/FS-reflection.md");
    fs::write(
        &file,
        concat!(
            "# FS-reflection: JVM reflection\n\n",
            "Resolved only where the Lean engine (argus.spec.md \u{a7}5.1) can say so.\n\n",
            "## 1. Scope\n\n",
            "The call graph (argus.spec.md \u{a7}4.5) bounds what is resolved.\n\n",
            "## 2. Limits\n\n",
            "As graph-analysis.md \u{a7}10 requires; see \u{a7}2 above.\n",
        ),
    )
    .expect("write declaration");

    let before = run(&root, &["check"]);
    assert_eq!(before.status.code(), Some(1), "{}", stdout(&before));
    let mut lines = fs::read_to_string(&file)
        .expect("read declaration")
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut followed = Vec::new();
    for finding in stdout(&before).lines() {
        let Some((location, message)) = finding.split_once(": error: ") else {
            continue;
        };
        let Some(rest) = message.strip_prefix("local section citation ") else {
            continue;
        };
        let token = rest.split_once(';').expect("the token ends at `;`").0;
        let line = location
            .rsplit_once(':')
            .and_then(|(_, line)| line.parse::<usize>().ok())
            .expect("a located finding");
        let remedy = the_written_remedy(message);
        let text = &mut lines[line - 1];
        assert!(text.contains(token), "line {line} holds {token}: {text:?}");
        *text = text.replacen(token, &remedy, 1);
        followed.push(format!("{token} -> {remedy}"));
    }
    assert_eq!(followed.len(), 4, "four owned sites: {}", stdout(&before));
    fs::write(&file, lines.join("\n") + "\n").expect("apply the remedies");

    let after = run(&root, &["check"]);
    assert_eq!(
        (after.status.code(), stdout(&after).as_str()),
        (Some(0), "success\n"),
        "§FS-check.3.24.3: each finding's own remedy clears its site; followed {followed:?}"
    );
    let _ = fs::remove_dir_all(root);
}
