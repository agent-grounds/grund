//! Same-file advice preserves OwnFile pairing (§FS-declarations.stubs.repair).

use super::support::*;
use serde_json::Value;

const DUPLICATE_PREFIX: &str =
    "duplicate declaration of AR-cache (also declared at docs/architecture/cache.md:3)";
const AMBIGUOUS_PREFIX: &str = "ambiguous ID: AR-cache (declared at docs/architecture/cache.md:1, docs/architecture/cache.md:3)";
const CANNOT_PAIR: &str = "linking to the same file will not pair it";

#[test]
fn missing_stub_same_file_home_explains_why_linking_cannot_pair() {
    let repo = Repo::new();
    repo.same_file_home("src/cache.rs");
    let findings = check_with_text(&repo);
    assert_eq!(findings.len(), 2, "{findings:?}");
    let expected_sites = sites_at(&[(SAME_FILE, 1), (SAME_FILE, 3)]);
    check_shape(&findings[0], "duplicate", SAME_FILE, expected_sites.clone());
    check_shape(&findings[1], "broken-stub", SAME_FILE, Value::Null);
    let suffix = message(&findings[0])
        .strip_prefix(DUPLICATE_PREFIX)
        .expect("complete original duplicate prefix");
    assert_eq!(
        suffix,
        format!(
            "; broken stub: {SAME_FILE}:1 links to missing src/cache.rs; \
         AR-cache is declared at {SAME_FILE}:3; stub {SAME_FILE}:1 \
         shares the real home's file; {CANNOT_PAIR}"
        )
    );
    own_file_connection(message(&findings[1]));

    // Selection must retain complete records, even when the companion is hidden.
    for finding in &findings {
        let code = finding["code"].as_str().unwrap();
        let output = repo.run(&["check", "--only", code, "--format=json"], 1);
        assert!(output.stderr.is_empty());
        assert_eq!(records(&output.stdout), vec![finding.clone()]);
        let text = repo.run(&["check", "--only", code], 1);
        assert!(text.stderr.is_empty());
        assert_eq!(
            utf8(&text.stdout),
            format!("{SAME_FILE}:1: error: {}\n", message(finding))
        );
    }
    for args in [
        vec!["AR-cache"],
        vec!["AR-cache", "--toc"],
        vec!["AR-cache.1"],
    ] {
        let query = query_with_text(&repo, &args, expected_sites.clone());
        assert_eq!(message(&query), format!("{AMBIGUOUS_PREFIX}{suffix}"));
    }
    // §FS-refs.4.1.1: `refs` sets the stub aside and warns with the same clause.
    let refs = repo.run(&["refs", "AR-cache"], 0);
    assert_eq!(utf8(&refs.stdout), "src/main.rs:1: \u{a7}AR-cache\n");
    assert_eq!(
        utf8(&refs.stderr),
        format!("warning: {}\n", suffix.strip_prefix("; ").unwrap())
    );
}

#[test]
fn own_file_link_keeps_duplicate_and_ambiguity_without_missing_target_advice() {
    let repo = Repo::new();
    repo.same_file_home("src/cache.rs");
    // Follow the formerly offered advice. This is OwnFile, not a healthy pair.
    repo.write_same_file_home("cache.md");
    let findings = check_with_text(&repo);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let expected_sites = sites_at(&[(SAME_FILE, 1), (SAME_FILE, 3)]);
    check_shape(&findings[0], "duplicate", SAME_FILE, expected_sites.clone());
    assert_eq!(message(&findings[0]), DUPLICATE_PREFIX);
    for args in [vec!["AR-cache"], vec!["refs", "AR-cache"]] {
        let query = query_with_text(&repo, &args, expected_sites.clone());
        assert_eq!(message(&query), AMBIGUOUS_PREFIX);
    }
}

#[test]
fn missing_stub_mixed_files_scope_advice_to_the_separate_file_stub() {
    let repo = Repo::new();
    repo.same_file_home("src/cache.rs");
    repo.write(STUB, "# AR-cache: [src/cache.rs](src/cache.rs)\n");
    let findings = check_with_text(&repo);
    assert_eq!(findings.len(), 3, "{findings:?}");
    let expected_sites = sites_at(&[(STUB, 1), (SAME_FILE, 1), (SAME_FILE, 3)]);
    check_shape(&findings[0], "duplicate", STUB, expected_sites.clone());
    check_shape(&findings[1], "broken-stub", STUB, Value::Null);
    check_shape(&findings[2], "broken-stub", SAME_FILE, Value::Null);
    let prefix = "duplicate declaration of AR-cache (also declared at docs/architecture/cache.md:1, docs/architecture/cache.md:3)";
    let suffix = message(&findings[0])
        .strip_prefix(prefix)
        .expect("complete original duplicate prefix");
    assert_eq!(
        suffix,
        format!(
            "; broken stub: {STUB}:1 links to missing src/cache.rs; \
         broken stub: {SAME_FILE}:1 links to missing src/cache.rs; \
         AR-cache is declared at {SAME_FILE}:3; \
         point the stub link at {STUB}:1 to that real home; \
         stub {SAME_FILE}:1 shares the real home's file; {CANNOT_PAIR}"
        )
    );
    site_free_connection(message(&findings[1]), MISSING, &[SAME_FILE]);
    own_file_connection(message(&findings[2]));
    let selected = repo.run(&["check", "--only", "broken-stub", "--format=json"], 1);
    assert!(selected.stderr.is_empty());
    assert_eq!(records(&selected.stdout), findings[1..]);
    let text = repo.run(&["check", "--only", "broken-stub"], 1);
    assert!(text.stderr.is_empty());
    assert_eq!(
        utf8(&text.stdout),
        format!(
            "{STUB}:1: error: {}\n{SAME_FILE}:1: error: {}\n",
            message(&findings[1]),
            message(&findings[2])
        )
    );
    let query = query_with_text(&repo, &["AR-cache"], expected_sites);
    assert_eq!(
        message(&query),
        format!(
            "ambiguous ID: AR-cache (declared at {STUB}:1, {SAME_FILE}:1, {SAME_FILE}:3){suffix}"
        )
    );
    // §FS-refs.4.1.1: `refs` sets both stubs aside, one warning each, each clause as
    // the repair writes it for that stub alone beside the one real home.
    let refs = repo.run(&["refs", "AR-cache"], 0);
    assert_eq!(utf8(&refs.stdout), "src/main.rs:1: \u{a7}AR-cache\n");
    assert_eq!(
        utf8(&refs.stderr),
        format!(
            "warning: broken stub: {STUB}:1 links to missing src/cache.rs; \
             AR-cache is declared at {SAME_FILE}:3; point the stub link there\n\
             warning: broken stub: {SAME_FILE}:1 links to missing src/cache.rs; \
             AR-cache is declared at {SAME_FILE}:3; stub {SAME_FILE}:1 \
             shares the real home's file; {CANNOT_PAIR}\n"
        )
    );
}

fn message(finding: &Value) -> &str {
    finding["message"].as_str().unwrap()
}

fn own_file_connection(message: &str) {
    let suffix = message
        .strip_prefix(MISSING)
        .expect("original missing prefix");
    assert_eq!(
        suffix,
        format!(
            "; AR-cache has one real declaration home in this stub's own file; \
         {CANNOT_PAIR} (see the duplicate declaration finding for its location)"
        )
    );
    assert!(!suffix.contains("point the stub"));
    assert!(!suffix.contains(SAME_FILE) && !suffix.contains(":3"));
}

fn check_shape(finding: &Value, code: &str, path: &str, sites: Value) {
    shape(finding);
    assert_eq!(finding["code"], code);
    assert_eq!(finding["path"], path);
    assert_eq!(finding["line"], 1);
    assert_eq!(finding["sites"], sites);
}

fn check_with_text(repo: &Repo) -> Vec<Value> {
    let output = repo.run(&["check", "--format=json"], 1);
    assert!(output.stderr.is_empty());
    let findings = records(&output.stdout);
    let text = repo.run(&["check"], 1);
    assert!(text.stderr.is_empty());
    let expected = findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{}: error: {}\n",
                finding["path"].as_str().unwrap(),
                finding["line"],
                message(finding)
            )
        })
        .collect::<String>();
    assert_eq!(utf8(&text.stdout), expected);
    findings
}

fn query_with_text(repo: &Repo, args: &[&str], expected_sites: Value) -> Value {
    let text = repo.run(args, 1);
    assert!(text.stdout.is_empty());
    let mut args = args.to_vec();
    args.push("--format=json");
    let output = repo.run(&args, 1);
    assert!(output.stdout.is_empty());
    let findings = records(&output.stderr);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    shape(finding);
    assert_eq!(finding["code"], "ambiguous");
    assert_eq!(finding["path"], Value::Null);
    assert_eq!(finding["line"], Value::Null);
    assert_eq!(finding["sites"], expected_sites);
    assert_eq!(utf8(&text.stderr), format!("{}\n", message(finding)));
    finding.clone()
}
