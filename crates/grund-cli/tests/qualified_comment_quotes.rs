//! Black-box regression for §FS-citations.recognized.contexts: port grund.102's
//! control/subject check to the CLI, without unrelated fixture/index warnings.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, workspace: bool) -> Self {
        let root = std::env::temp_dir().join(format!("grund-778-{}-{name}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(root);
        fixture.write("grund.toml", &Self::config("root", workspace));
        if workspace {
            fixture.write(
                "api/grund.toml",
                &(Self::config("api", false) + "files = [\"spec.md\"]\n"),
            );
            fixture.write("api/spec.md", "# FS-entry: Entry\n\nSee \u{a7}FS-entry.\n");
        }
        fixture
    }

    fn config(project: &str, workspace: bool) -> String {
        format!(
            "grund_config_version = 2\nproject_name = {project:?}\n\
             {}\
             [schema.sources]\nlanguages = [\"markdown\", \"rust\", \"haskell\", \"python\"]\n\
             [schema.kinds.FS]\n",
            if workspace {
                "[workspace]\nmembers = [\"api\"]\n"
            } else {
                ""
            }
        )
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn run(&self, args: &[&str], code: i32) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "{args:?}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty(), "{:?}", output);
        String::from_utf8(output.stdout).unwrap()
    }

    fn reproducer(&self) {
        // Same five sites as the independently runnable intake reproducer.
        self.write(
            "src/a.rs",
            "// Control: \u{a7}api/FS-apos-control.\n\
             // Don't drop \u{a7}api/FS-apos-dont.\n\
             // Don't drop \u{a7}FS-apos-unqualified.\n\
             fn f() -> &'static str { \"x\" } // \u{a7}api/FS-apos-static\n",
        );
        self.write(
            "src/a.hs",
            "module A where\nf' = 1 -- \u{a7}api/FS-apos-prime\n",
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_report(actual: &str, expected: &[String]) {
    let missing = expected
        .iter()
        .filter(|line| !actual.lines().any(|found| found == line.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "missing diagnostics:\n{}\nactual report:\n{actual}",
        missing.join("\n")
    );
    assert_eq!(actual.lines().count(), expected.len(), "{actual}");
}

/// §FS-citations.recognized.contexts: each qualified comment is diagnosed like
/// the no-quote control; the marked unqualified apostrophe control stays live.
#[test]
fn qualified_comments_after_quotes_unknown_alias() {
    let fixture = Fixture::new("unknown-alias", false);
    fixture.reproducer();
    let actual = fixture.run(&["check"], 1);
    let expected = ["src/a.hs:2", "src/a.rs:1", "src/a.rs:2", "src/a.rs:4"]
        .map(|site| format!("{site}: error: unknown project alias api"));
    let mut expected = expected.to_vec();
    expected.push("src/a.rs:3: error: unknown reference FS-apos-unqualified".into());
    assert_report(&actual, &expected);
}

/// §FS-citations.recognized.contexts: configured aliases use the workspace
/// target verdict for every recovered site, rather than silently omitting it.
#[test]
fn qualified_comments_after_quotes_configured_alias() {
    let fixture = Fixture::new("configured-alias", true);
    fixture.reproducer();
    let actual = fixture.run(&["check"], 1);
    let expected = [
        ("src/a.hs:2", "prime"),
        ("src/a.rs:1", "control"),
        ("src/a.rs:2", "dont"),
        ("src/a.rs:4", "static"),
    ]
    .map(|(site, slug)| format!("{site}: error: unknown reference api/FS-apos-{slug}"));
    let mut expected = expected.to_vec();
    expected.push("src/a.rs:3: error: unknown reference FS-apos-unqualified".into());
    assert_report(&actual, &expected);
}

/// §FS-citations.recognized.contexts: genuine source strings, inline code and
/// carried literals stay excluded; Markdown's qualified forms stay live.
/// These preservation controls already pass before #778 is implemented.
#[test]
fn qualified_comment_quote_preservation() {
    for workspace in [false, true] {
        let fixture = Fixture::new(&format!("preservation-{workspace}"), workspace);
        fixture.write(
            "src/data.rs",
            "let s = \"\u{a7}api/FS-quoted\";\n\
             // `\u{a7}api/FS-inline`\n\
             let data = r#\"\n\
             // Don't read \u{a7}api/FS-carried\n\
             \"#;\n\
             // \"\u{a7}api/FS-comment-quoted\"\n\
             // \u{a7}api/FS-after\n",
        );
        fixture.write("src/data.py", "s = '\u{a7}api/FS-single-quoted'\n");
        fixture.write("notes.md", "Don't hide `\u{a7}api/FS-markdown`.\n");
        let actual = fixture.run(&["check"], 1);
        let expected = if workspace {
            vec![
                "notes.md:1: error: unknown reference api/FS-markdown; write <\u{a7}>api/FS-markdown if this is an illustration".into(),
                "src/data.rs:7: error: unknown reference api/FS-after".into(),
            ]
        } else {
            vec![
                "notes.md:1: error: unknown project alias api".into(),
                "src/data.rs:7: error: unknown project alias api".into(),
            ]
        };
        assert_report(&actual, &expected);
    }
}

/// §FS-fmt.2.3.1 and §FS-citations.recognized.string-literals keep the raw
/// single-quote toggle for formatter transforms and non-strict bare tokens.
#[test]
fn formatter_and_bare_tokens_keep_raw_quote_policy() {
    let fixture = Fixture::new("raw-quote-policy", false);
    fixture.write(
        "grund.toml",
        "grund_config_version = 1\n[reference]\nstrict = false\n\
         [id]\nformat = \"{kind}-{slug}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
         [scan]\nextensions = [\"md\", \"rs\", \"hs\"]\n",
    );
    fixture.write(
        "docs/FS-entry.md",
        "# FS-entry: Entry\n\nSee \u{a7}FS-entry.\n",
    );
    let rust = "// Don't rewrite $$FS-apos-trigger or read FS-apos-bare.\n\
                fn f() -> &'static str { \"x\" } // $$FS-static-trigger FS-static-bare\n";
    let haskell = "f' = 1 -- $$FS-prime-trigger FS-prime-bare\n";
    fixture.write("src/a.rs", rust);
    fixture.write("src/a.hs", haskell);
    fixture.write("src/control.rs", "// Read FS-plain-bare.\n");
    let check = fixture.run(&["check", "--ignore", "config-version-1"], 1);
    assert_report(
        &check,
        &["src/control.rs:1: error: unknown reference FS-plain-bare".into()],
    );
    fixture.write("src/control.rs", "// $$FS-entry\n");
    fixture.run(&["fmt", "--write"], 0);
    assert_eq!(
        fs::read_to_string(fixture.0.join("src/control.rs")).unwrap(),
        "// \u{a7}FS-entry\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("src/a.rs")).unwrap(),
        rust
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("src/a.hs")).unwrap(),
        haskell
    );
}
