//! This issue's eleven keys, typed one at a time over stdio, each answered for its
//! own text (§FS-lsp.responsiveness.4, §FS-lsp.1.6.2): in a Markdown note and in a
//! Java `//` comment, every key is a `didChange` followed by a `completion`, and
//! each completion offers exactly the declarations the prefix typed so far names.
//! A guard rather than a red test: today's server already answers each key for its
//! own text, only late, and an answer from an obsolete prefix must not pass after it
//! answers early.
use super::completion_support::*;

/// The keys, `\u{8}` for a backspace, as the issue's session types them.
const KEYS: [char; 11] = [
    '$', '$', 'F', 'S', '-', '0', '0', '2', '\u{8}', '\u{8}', 'Z',
];

const ALL_FS: &[&str] = &["FS-001-intro", "FS-002-login", "FS-020-logout"];

/// What each prefix offers: the declared IDs that start with what follows `$$`.
fn offered(typed: &str) -> &'static [&'static str] {
    match typed {
        "$" | "$$FS-0Z" => &[],
        "$$" => &[
            "AR-001-server",
            "FS-001-intro",
            "FS-002-login",
            "FS-020-logout",
        ],
        "$$F" | "$$FS" | "$$FS-" | "$$FS-0" => ALL_FS,
        "$$FS-00" => &["FS-001-intro", "FS-002-login"],
        "$$FS-002" => &["FS-002-login"],
        other => panic!("no expectation for {other:?}"),
    }
}

fn fixture() -> Fixture {
    let f = Fixture::new();
    for stale in ["FS-login", "FS-logout"] {
        std::fs::remove_file(f.0.join(format!("docs/functional-spec/{stale}.md"))).unwrap();
    }
    f.write(
        "grund.toml",
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{number}-{slug}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n\
         [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"java\"]\n\
         [fmt.cross_refs]\nenabled = false\n",
    );
    for (folder, id, title) in [
        ("functional-spec", "FS-001-intro", "Intro"),
        ("functional-spec", "FS-002-login", "Login"),
        ("functional-spec", "FS-020-logout", "Logout"),
        ("architecture", "AR-001-server", "Server"),
    ] {
        f.write(
            &format!("docs/{folder}/{id}.md"),
            &format!("# {id}: {title}\n\nThe lead.\n"),
        );
    }
    f.write("src/Main.java", "class Main {\n}\n");
    f
}

/// Type the eleven keys at the end of line `line` of `rel`, whose text `around`
/// builds from the typed prefix, and hold each completion to what its prefix offers.
fn type_eleven_keys(rel: &str, line: usize, around: fn(&str) -> String) {
    let f = fixture();
    let path = f.0.join(rel);
    let mut s = Session::new(&f.0);
    let mut typed = String::new();
    for key in KEYS {
        if key == '\u{8}' {
            typed.pop();
        } else {
            typed.push(key);
        }
        let text = around(&typed);
        let cursor = text.lines().nth(line).unwrap().len();
        let items = s.completion(&path, &text, line, cursor);
        let mut labels: Vec<_> = items
            .iter()
            .map(|item| item["label"].as_str().unwrap().to_string())
            .collect();
        labels.sort();
        assert_eq!(
            labels,
            offered(&typed),
            "{rel}: after typing {key:?} the prefix is {typed:?}"
        );
    }
}

#[test]
fn each_of_eleven_keys_in_markdown_prose_is_completed_for_its_own_prefix() {
    type_eleven_keys("docs/notes.md", 2, |typed| {
        format!("# Notes\n\nPlain prose, see {typed}\n")
    });
}

#[test]
fn each_of_eleven_keys_in_a_java_comment_is_completed_for_its_own_prefix() {
    type_eleven_keys("src/Main.java", 1, |typed| {
        format!("class Main {{\n    // Plain prose, see {typed}\n}}\n")
    });
}
