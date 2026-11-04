//! What is refused at `<home>/<ID>.md` when the scan's walk found no declaring file
//! (§FS-fetch.5.2), which the e2e goldens cannot pin: the refusal prints the absolute path
//! and a `command.cwd` case's stderr golden has no spelling for it that holds on another
//! machine. A guard here holds today and after the fix, on the exit and the bytes; an
//! ignored test is what the message must become.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use support::*;

const TARGET: &str = "docs/tickets/TICKET-1234.md";
const OLD: &str = "# TICKET-1234: The snapshot a first fetch wrote\n\nThe old body.\n";
const BESIDE: &str = "# TICKET-1234: Requested\n\nBody.\n\n# TICKET-1235: Other\n\nBody.\n";

const ORDINARY: &str = "[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n";
const MD_AS_JSON: &str = concat!(
    "[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n\n",
    "[schema.sources.extensions]\nmd = \"json\"\n"
);
const NO_LANGUAGE: &str = "[schema.sources]\nlanguages = [\"json\"]\n";
const FILE_EXCLUDED: &str = concat!(
    "[schema.sources]\nlanguages = [\"markdown\", \"json\"]\n",
    "exclude = [\"docs/tickets/TICKET-1234.md\"]\n"
);

/// The configurations whose walk does not admit the target, each with a name for its fixture.
const UNREAD: [(&str, &str); 3] = [
    ("md-as-json", MD_AS_JSON),
    ("no-language", NO_LANGUAGE),
    ("file-excluded", FILE_EXCLUDED),
];

/// Those and the one whose walk does.
const EVERY: [(&str, &str); 4] = [
    ("ordinary", ORDINARY),
    ("md-as-json", MD_AS_JSON),
    ("no-language", NO_LANGUAGE),
    ("file-excluded", FILE_EXCLUDED),
];

fn project(at: &Path, sources: &str) {
    write(
        at,
        "grund.toml",
        &format!(
            "grund_config_version = 2\nproject_name = \"tickets\"\n\n{sources}\n\
             [schema.kinds.TICKET]\nfolders = [\"docs/tickets\"]\n\
             id_format = \"{{kind}}-{{number}}\"\nindex = false\n\
             fetch = \"scripts/fetch-ticket\"\n"
        ),
    );
    executable(
        at,
        "scripts/fetch-ticket",
        "#!/bin/sh\nprintf '# %s: The fetched ticket\\n\\nThe fetched body.\\n' \"$1\"\n",
    );
    fs::create_dir_all(at.join("docs/tickets")).expect("create the home");
}

/// A project below `outside/`'s sibling, with a link at the target leading out of it to a
/// file that declares the requested ID. Returns the project and the file the link leads to.
fn linked_out(name: &str, sources: &str) -> (PathBuf, PathBuf) {
    let fixture = root(name);
    let at = fixture.join("project");
    project(&at, sources);
    write(&fixture, "outside/TICKET-1234.md", OLD);
    symlink("../../../outside/TICKET-1234.md", at.join(TARGET)).expect("link the target");
    (at, fixture.join("outside/TICKET-1234.md"))
}

/// The link is still the same link and what it leads to still holds its bytes.
fn assert_link_intact(at: &Path, outside: &Path, context: &str) {
    let link = at.join(TARGET);
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "{context}: the link was replaced"
    );
    assert_eq!(
        fs::read_link(&link).unwrap(),
        Path::new("../../../outside/TICKET-1234.md"),
        "{context}: the link was repointed"
    );
    assert_eq!(
        fs::read_to_string(outside).unwrap(),
        OLD,
        "{context}: wrote through the link"
    );
}

#[test]
fn a_file_declaring_another_id_is_refused_for_a_different_id() {
    for (name, sources) in EVERY {
        let at = root(&format!("read-other-id-{name}"));
        project(&at, sources);
        write(
            &at,
            TARGET,
            "# TICKET-8: Another ticket\n\nNot the one requested.\n",
        );
        let before = tree(&at);
        let output = run(&at, &["fetch", "TICKET-1234"]);
        assert_code(&output, 2, name);
        assert_eq!(tree(&at), before, "{name}: the refusal moved bytes");
        let message = stderr(&output);
        assert!(
            message.contains("already exists for a different ID"),
            "{name}: {message}"
        );
    }
}

#[test]
fn a_file_declaring_nothing_stays_refused() {
    for (name, sources) in EVERY {
        for (shape, body) in [("empty", ""), ("prose", "Notes kept by hand.\n")] {
            let at = root(&format!("read-nothing-{shape}-{name}"));
            project(&at, sources);
            write(&at, TARGET, body);
            let before = tree(&at);
            assert_code(&run(&at, &["fetch", "TICKET-1234"]), 2, name);
            assert_eq!(
                tree(&at),
                before,
                "{name}, {shape}: the refusal moved bytes"
            );
        }
    }
}

#[test]
fn an_unread_file_holding_the_id_beside_other_content_stays_refused() {
    for (name, sources) in UNREAD {
        let at = root(&format!("read-beside-{name}"));
        project(&at, sources);
        write(&at, TARGET, BESIDE);
        let before = tree(&at);
        assert_code(&run(&at, &["fetch", "TICKET-1234"]), 2, name);
        assert_eq!(tree(&at), before, "{name}: the refusal moved bytes");
    }
}

#[test]
fn an_unread_file_that_is_not_utf8_stays_refused() {
    for (name, sources) in UNREAD {
        let at = root(&format!("read-not-utf8-{name}"));
        project(&at, sources);
        fs::write(at.join(TARGET), b"\xff\xfe\n").unwrap();
        let before = tree(&at);
        assert_code(&run(&at, &["fetch", "TICKET-1234"]), 2, name);
        assert_eq!(tree(&at), before, "{name}: the refusal moved bytes");
    }
}

#[test]
fn a_link_the_walk_does_not_read_stays_refused_and_intact() {
    for (name, sources) in UNREAD {
        let (at, outside) = linked_out(&format!("read-link-{name}"), sources);
        assert_code(&run(&at, &["fetch", "TICKET-1234"]), 2, name);
        assert_link_intact(&at, &outside, name);
    }
}

#[test]
fn a_directory_at_the_path_stays_refused() {
    for (name, sources) in [("ordinary", ORDINARY), ("md-as-json", MD_AS_JSON)] {
        let at = root(&format!("read-directory-{name}"));
        project(&at, sources);
        fs::create_dir(at.join(TARGET)).unwrap();
        assert_code(&run(&at, &["fetch", "TICKET-1234"]), 2, name);
        assert!(
            at.join(TARGET).is_dir(),
            "{name}: the directory was replaced"
        );
    }
}

/// §FS-fetch.5.1: this is the walk's link, not the read's, and its path is replaced where
/// the link above is refused. Pinned so the difference reads as chosen.
#[test]
fn a_link_the_walk_reads_through_is_replaced_and_what_it_led_to_left_alone() {
    let (at, outside) = linked_out("read-link-walked", ORDINARY);
    assert_code(&run(&at, &["fetch", "TICKET-1234"]), 0, "walked link");
    let link = at.join(TARGET);
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_file());
    assert!(
        fs::read_to_string(&link)
            .unwrap()
            .contains("The fetched body.")
    );
    assert_eq!(fs::read_to_string(&outside).unwrap(), OLD);
}

#[test]
#[ignore = "red until #1185 says what the file holds; implement removes this"]
fn a_file_declaring_nothing_is_told_it_does_not_declare_the_id() {
    for (name, sources) in EVERY {
        for (shape, body) in [("empty", ""), ("prose", "Notes kept by hand.\n")] {
            let at = root(&format!("words-nothing-{shape}-{name}"));
            project(&at, sources);
            write(&at, TARGET, body);
            let message = stderr(&run(&at, &["fetch", "TICKET-1234"]));
            assert!(
                message.contains("already exists and does not declare TICKET-1234"),
                "{name}, {shape}: {message}"
            );
        }
    }
}

#[test]
#[ignore = "red until #1185 says what the unread file holds; implement removes this"]
fn an_unread_file_holding_the_id_beside_other_content_is_told_so() {
    for (name, sources) in UNREAD {
        let at = root(&format!("words-beside-{name}"));
        project(&at, sources);
        write(&at, TARGET, BESIDE);
        let message = stderr(&run(&at, &["fetch", "TICKET-1234"]));
        assert!(
            message.contains("contains the requested ID beside other content"),
            "{name}: {message}"
        );
    }
}

#[test]
#[ignore = "red until #1185 says what stands at the path; implement removes this"]
fn a_refused_link_is_told_it_is_not_a_regular_file() {
    for (name, sources) in UNREAD {
        let (at, _outside) = linked_out(&format!("words-link-{name}"), sources);
        let message = stderr(&run(&at, &["fetch", "TICKET-1234"]));
        assert!(
            message.contains("is not a regular file"),
            "{name}: {message}"
        );
    }
}

#[test]
#[ignore = "red until #1185 says what stands at the path; implement removes this"]
fn a_refused_directory_is_told_it_is_not_a_regular_file() {
    for (name, sources) in [("ordinary", ORDINARY), ("md-as-json", MD_AS_JSON)] {
        let at = root(&format!("words-directory-{name}"));
        project(&at, sources);
        fs::create_dir(at.join(TARGET)).unwrap();
        let message = stderr(&run(&at, &["fetch", "TICKET-1234"]));
        assert!(
            message.contains("is not a regular file"),
            "{name}: {message}"
        );
    }
}
