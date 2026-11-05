//! `grund fetch` refuses a `file` home the text scan reads as source (§FS-fetch.4.2), and
//! `check` stops naming that command there (§FS-check.3.1.3.1). The e2e cases pin each
//! refusal's exact bytes; these pin what a golden cannot: that every way out the refusal
//! and the tail offer ends green, and that each comment form is refused alike.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::{assert_code, executable, root, run, stderr, stdout, tree, write};

const MARK: &str = "fetcher-ran";
const DOCSTRING: &str = "\"\"\"FS-a: Alpha\n\nLocal body.\n\"\"\"\nx = 1\n";
const FILE_FETCHER: &str =
    "#!/bin/sh\ntouch fetcher-ran\nprintf '## %s: Fetched\\n\\nFetched body.\\n' \"$1\"\n";
const FOLDER_FETCHER: &str =
    "#!/bin/sh\ntouch fetcher-ran\nprintf '# %s: Fetched\\n\\nFetched body.\\n' \"$1\"\n";
const VALUE_FETCHER: &str = "#!/bin/sh\nprintf '## %s: Fetched\\n\\n### 1. 99\\n' \"$1\"\n";
const VALUE_FOLDER_FETCHER: &str = "#!/bin/sh\nprintf '# %s: Fetched\\n\\n## 1. 99\\n' \"$1\"\n";
const DOCSTRING_FETCHER: &str = "#!/bin/sh\ntouch fetcher-ran\nprintf '## %s: Alpha\\n\\n\
     \"\"\"%s: Alpha\\n\\nFetched body citing \u{a7}DOC-one.\\n\"\"\"\\n' \"$1\" \"$1\"\n";
const LEFT_BY_THE_OLD_FETCH: &str =
    "x = 1\n\n## FS-a: Alpha\n\n\"\"\"FS-a: Alpha\n\nFetched body citing \u{a7}DOC-one.\n\"\"\"\n";

fn config(place: &str, languages: &str, fetch: bool) -> String {
    let fetch = if fetch {
        "fetch = \"scripts/fetch-fs\"\n"
    } else {
        ""
    };
    format!(
        "grund_config_version = 2\nproject_name = \"r\"\n\n[schema.sources]\n\
         languages = [{languages}]\n\n[schema.kinds.FS]\n{place}\n{fetch}"
    )
}

fn python_project(name: &str) -> PathBuf {
    let dir = root(name);
    let place = "files = [\"fs.py\"]";
    write(
        &dir,
        "grund.toml",
        &config(place, "\"markdown\", \"python\"", true),
    );
    write(&dir, "fs.py", DOCSTRING);
    executable(&dir, "scripts/fetch-fs", FILE_FETCHER);
    dir
}

/// The refusal §FS-fetch.4.2 prints: exit 2, nothing on stdout, nothing changed, and the
/// integration never ran.
fn assert_refused(dir: &Path, id: &str, home: &str) {
    let before = tree(dir);
    let output = run(dir, &["fetch", id]);
    assert_code(&output, 2, home);
    assert_eq!(stdout(&output), "", "{home}: stdout must stay empty");
    assert_eq!(
        stderr(&output),
        format!(
            "error: kind `FS` cannot fetch into {home}: a file read as source holds no Markdown; \
             move the kind and its declarations to a Markdown file or a folder, or remove its \
             `fetch`\n"
        ),
    );
    assert_eq!(tree(dir), before, "{home}: the refusal must change nothing");
}

/// Fetch and check are green, the integration ran, and a read returns what it wrote.
fn assert_fetch_lands(dir: &Path) {
    assert_code(&run(dir, &["fetch", "FS-a"]), 0, "fetch after the move");
    assert!(dir.join(MARK).exists(), "the integration must have run");
    assert_code(&run(dir, &["check"]), 0, "check after the fetch");
    let shown = run(dir, &["FS-a", "--full"]);
    assert_code(&shown, 0, "show after the fetch");
    assert!(
        stdout(&shown).contains("Fetched body."),
        "a read must return what fetch wrote: {:?}",
        stdout(&shown)
    );
}

#[test]
fn moving_the_kind_and_its_declarations_to_a_markdown_file_is_a_way_out() {
    let dir = python_project("source-home-way-out-markdown");
    assert_refused(&dir, "FS-a", "fs.py");

    let languages = "\"markdown\", \"python\"";
    write(
        &dir,
        "grund.toml",
        &config("files = [\"fs.md\"]", languages, true),
    );
    write(&dir, "fs.py", "x = 1\n");
    assert_fetch_lands(&dir);
    assert_eq!(fs::read_to_string(dir.join("fs.py")).unwrap(), "x = 1\n");
}

#[test]
fn moving_the_kind_and_its_declarations_to_a_folder_is_a_way_out() {
    let dir = python_project("source-home-way-out-folder");
    assert_refused(&dir, "FS-a", "fs.py");

    let languages = "\"markdown\", \"python\"";
    write(
        &dir,
        "grund.toml",
        &config("folders = [\"src\"]", languages, true),
    );
    fs::remove_file(dir.join("fs.py")).unwrap();
    write(&dir, "src/fs.py", "x = 1\n");
    write(
        &dir,
        "src/README.md",
        "# Facts\n\n- [\u{a7}FS-a](FS-a.md)\n",
    );
    executable(&dir, "scripts/fetch-fs", FOLDER_FETCHER);
    assert_fetch_lands(&dir);
    assert_eq!(
        fs::read_to_string(dir.join("src/fs.py")).unwrap(),
        "x = 1\n"
    );
}

#[test]
fn removing_fetch_is_a_way_out() {
    let dir = python_project("source-home-way-out-no-fetch");
    assert_refused(&dir, "FS-a", "fs.py");

    let languages = "\"markdown\", \"python\"";
    write(
        &dir,
        "grund.toml",
        &config("files = [\"fs.py\"]", languages, false),
    );
    assert_code(&run(&dir, &["check"]), 0, "check without fetch");
    let shown = run(&dir, &["FS-a", "--full"]);
    assert_code(&shown, 0, "show without fetch");
    assert_eq!(stdout(&shown), "Local body.\n");
    let fetch = run(&dir, &["fetch", "FS-a"]);
    assert_code(&fetch, 2, "fetch without fetch");
    assert_eq!(
        stderr(&fetch),
        "error: kind `FS` has no fetch integration\n"
    );
}

#[test]
fn declaring_it_in_that_source_file_answers_the_check_tail() {
    let dir = python_project("source-home-tail-way-through");
    let languages = "\"markdown\", \"python\"";
    let page = "\n[schema.kinds.page]\ncitable = false\nfiles = [\"docs/offer.md\"]\n";
    let place = "files = [\"fs.py\"]";
    write(
        &dir,
        "grund.toml",
        &format!("{}{page}", config(place, languages, true)),
    );
    write(
        &dir,
        "docs/offer.md",
        "# Offer\n\nThe fact is \u{a7}FS-other.\n",
    );

    let dangling = run(&dir, &["check"]);
    assert_code(&dangling, 1, "check before the declaration");
    assert!(
        stdout(&dangling).contains("no snapshot in fs.py — declare it in that source file"),
        "the tail must name what works: {:?}",
        stdout(&dangling)
    );

    write(
        &dir,
        "fs.py",
        "\"\"\"FS-a: Alpha\n\nLocal body.\n\"\"\"\n\n\ndef f():\n    \
         \"\"\"FS-other: Other\n\n    Other body.\n    \"\"\"\n",
    );
    assert_code(&run(&dir, &["check"]), 0, "check after the declaration");
}

/// No comment in a source file is a valid value declaration, so §FS-check.3.1.3.1 sends a
/// value kind to Markdown instead, and each home the tail names ends green. A value file
/// home must exist before its first snapshot (§FS-fetch.4.1); a folder needs its index
/// (§FS-check.3.18) and takes an H1 (§FS-fetch.3).
#[test]
fn moving_a_value_kind_to_markdown_answers_the_check_tail() {
    let index = "# Facts\n\n- [\u{a7}FS-other](FS-other.md)\n";
    let homes = [
        ("files = [\"fs.md\"]", "fs.md", "", VALUE_FETCHER),
        (
            "folders = [\"src\"]",
            "src/README.md",
            index,
            VALUE_FOLDER_FETCHER,
        ),
    ];
    for (place, made, made_body, fetcher) in homes {
        let dir = root(&format!(
            "source-home-value-tail-{}",
            made.replace('/', "-")
        ));
        let config_at = |place: &str| {
            format!(
                "{}form = \"value\"\n\n[schema.kinds.page]\ncitable = false\n\
                 files = [\"docs/offer.md\"]\n",
                config(place, "\"markdown\", \"python\"", true)
            )
        };
        write(&dir, "grund.toml", &config_at("files = [\"fs.py\"]"));
        write(&dir, "fs.py", "x = 1\n");
        write(
            &dir,
            "docs/offer.md",
            "# Offer\n\nThe fact is \u{a7}FS-other.\n",
        );
        executable(&dir, "scripts/fetch-fs", fetcher);

        let dangling = run(&dir, &["check"]);
        assert_code(&dangling, 1, "check before the move");
        assert!(
            stdout(&dangling)
                .contains("no snapshot in fs.py — move the kind to a Markdown file or folder"),
            "the tail must name what works: {:?}",
            stdout(&dangling)
        );

        write(&dir, "grund.toml", &config_at(place));
        write(&dir, made, made_body);
        assert_code(&run(&dir, &["fetch", "FS-other"]), 0, place);
        assert_code(&run(&dir, &["check"]), 0, place);
        assert_eq!(fs::read_to_string(dir.join("fs.py")).unwrap(), "x = 1\n");
    }
}

/// Every read of the tree `LEFT_BY_THE_OLD_FETCH` holds, which the refusal does not move.
fn assert_docstring_reads(dir: &Path) {
    let listed = stdout(&run(dir, &["list"]));
    assert!(
        listed
            .lines()
            .any(|line| line.starts_with("FS-a ") && line.contains("fs.py:5 ")),
        "list must name the docstring's line: {listed:?}"
    );
    let shown = run(dir, &["FS-a", "--full"]);
    assert_code(&shown, 0, "show");
    assert_eq!(stdout(&shown), "Fetched body citing \u{a7}DOC-one.\n");
    let refs = stdout(&run(dir, &["refs", "DOC-one"]));
    assert!(refs.contains("fs.py:7"), "the body cites: {refs:?}");
    assert_code(&run(dir, &["check"]), 0, "check");
}

/// The one tree whose old success was real (§DF-fetch-source-home.3.6): an integration printing
/// a docstring under its H2 left a `fs.py` the scan reads. The refusal takes its fetch and none
/// of its reads, and each way out the record names ends green.
#[test]
fn a_fetched_docstring_keeps_its_reads_and_each_way_out_ends_green() {
    let dir = root("source-home-fetched-docstring");
    let config_at = |place: &str, fetch: bool| {
        format!(
            "{}\n[schema.kinds.DOC]\nfiles = [\"docs/doc.md\"]\n",
            config(place, "\"markdown\", \"python\"", fetch)
        )
    };
    write(&dir, "grund.toml", &config_at("files = [\"fs.py\"]", true));
    write(&dir, "docs/doc.md", "# DOC-one: Doc\n\nCites \u{a7}FS-a.\n");
    write(&dir, "fs.py", LEFT_BY_THE_OLD_FETCH);
    executable(&dir, "scripts/fetch-fs", DOCSTRING_FETCHER);
    assert_docstring_reads(&dir);
    assert_refused(&dir, "FS-a", "fs.py");

    write(&dir, "grund.toml", &config_at("files = [\"fs.py\"]", false));
    assert_docstring_reads(&dir);

    write(&dir, "grund.toml", &config_at("files = [\"fs.md\"]", true));
    write(&dir, "fs.py", "x = 1\n");
    assert_code(
        &run(&dir, &["fetch", "FS-a"]),
        0,
        "fetch, integration as it was",
    );
    assert_code(&run(&dir, &["check"]), 0, "check, integration as it was");
    assert_eq!(
        stdout(&run(&dir, &["FS-a", "--full"])),
        "\"\"\"FS-a: Alpha\n\nFetched body citing \u{a7}DOC-one.\n\"\"\"\n",
        "the integration has to change with the move, or the read returns the docstring's form"
    );

    fs::remove_file(dir.join(MARK)).unwrap();
    executable(&dir, "scripts/fetch-fs", FILE_FETCHER);
    assert_fetch_lands(&dir);
    assert_eq!(fs::read_to_string(dir.join("fs.py")).unwrap(), "x = 1\n");
}

/// The comment forms no e2e case holds. Each home declares the requested ID its own way,
/// and none of them is a Markdown reader's business.
#[test]
fn every_comment_form_is_refused_alike() {
    let forms = [
        (
            "python",
            "fs.py",
            "x = 1\n\ndef f():\n    \"\"\"FS-a: Alpha\n\n    Local body.\n    \"\"\"\n    return x\n",
        ),
        (
            "go",
            "fs.go",
            "package a\n\n// FS-a: Alpha\n//\n// Local body.\nfunc X() {}\n",
        ),
        (
            "javascript",
            "fs.js",
            "/** FS-a: Alpha\n *\n * Local body.\n */\nexport const x = 1;\n",
        ),
        (
            "ruby",
            "fs.rb",
            "# FS-a: Alpha\n#\n# Local body.\ndef x\nend\n",
        ),
    ];
    for (language, home, body) in forms {
        let dir = root(&format!("source-home-form-{language}"));
        write(
            &dir,
            "grund.toml",
            &config(
                &format!("files = [\"{home}\"]"),
                &format!("\"markdown\", \"{language}\""),
                true,
            ),
        );
        write(&dir, home, body);
        executable(&dir, "scripts/fetch-fs", FILE_FETCHER);
        let shown = run(&dir, &["FS-a", "--full"]);
        assert_eq!(
            stdout(&shown),
            "Local body.\n",
            "{language}: the home must declare the ID the way the scan reads it"
        );
        assert_refused(&dir, "FS-a", home);
    }
}
