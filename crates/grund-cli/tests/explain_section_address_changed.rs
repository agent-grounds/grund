//! grund#1027: a cited section moved on purpose, and the reader who asks
//! `grund config explain` about the error it earns. The answer gives the
//! error's own remedy, with its slots unfilled, and what the lock compares and
//! what writes it (§FS-config-explain.4), because both are the code's catalog
//! cells (§FS-errors.5.5.1) and the message carries the same *Fix* cell filled
//! in (§FS-section-lock.checks.section-address-changed). The tree is the
//! issue's: Email moves from `FS-login.format.1` to `FS-login.input.1` while
//! `src/auth/normalize.py` still cites the old address.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"login\"\n\n\
                      [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n";

const BEFORE: &str = "# Requirements\n\n## FS-login: Login\n\nWhat a login accepts.\n\n\
                      ### format: Input format\n\n#### .1: Email\n\n\
                      Surrounding whitespace is trimmed and the address is lower-cased.\n\n\
                      #### .2: Password\n\n\
                      The password is used exactly as typed; nothing is trimmed.\n";

/// Email moved to a new `input` section; Password moved up to `format.1`, line 17.
const AFTER: &str = "# Requirements\n\n## FS-login: Login\n\nWhat a login accepts.\n\n\
                     ### input: Input handling\n\n#### .1: Email\n\n\
                     Surrounding whitespace is trimmed and the address is lower-cased.\n\n\
                     ### format: Password handling\n\nWhat the password must be.\n\n\
                     #### .1: Password\n\n\
                     The password is used exactly as typed; nothing is trimmed.\n";

const SOURCE: &str = "\"\"\"Normalize login input.\"\"\"\n\n\n\
                      def normalize(email):\n    # \u{a7}FS-login.format.1\n\
                      \x20   return email.strip().lower()\n";

/// The check message's words before its remedy, as grund 0.20 prints them.
const FINDING: &str = "requirements.md:17: error: FS-login.format.1 is cited, and what \
                       .grund/sections/FS-login.lock recorded for it now sits at \
                       FS-login.input.1; ";

/// What the issue's reader is told today, and must still be told: the remedy.
const REMEDY: &str = "move it back, cite FS-login.input.1 instead, or leave a pointer to it \
                      at FS-login.format.1; if FS-login.input.1 is a copy, edit the copy or \
                      cite it from FS-login.format.1";

/// grund#1022: what the message appends after its remedy, the line that still
/// cites the old address (§FS-section-lock.checks.section-address-changed).
const CITED: &str = "; FS-login.format.1 is cited at src/auth/normalize.py:5";

const FIX: &str = "Fix: move it back, cite {new address} instead, or leave a pointer to it \
                   at {cited address}; if {new address} is a copy, edit the copy or cite it \
                   from {cited address}";

const NOTE: &str = "Note: check compares the text at a cited address with the fingerprint \
                    .grund/sections/<ID>.lock recorded for it, and finds that fingerprint at \
                    another address of the declaration. Only grund fmt --write writes a lock \
                    (grund config explain unrecorded-section-citation says where to run it). \
                    It never approves a move: the one thing it records at the old address is \
                    a pointer to the new one. Do not edit a lock by hand.";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Tree {
    root: PathBuf,
}

impl Tree {
    /// The issue's tree, locked by `fmt --write` and passing, then restructured.
    fn moved() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1027-moved-email-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/auth")).unwrap();
        let tree = Self { root };
        fs::write(tree.root.join("grund.toml"), CONFIG).unwrap();
        fs::write(tree.root.join("requirements.md"), BEFORE).unwrap();
        fs::write(tree.root.join("src/auth/normalize.py"), SOURCE).unwrap();
        tree.run(&["fmt", "--write"], 0);
        assert!(tree.root.join(".grund/sections/FS-login.lock").is_file());
        tree.run(&["check"], 0);
        fs::write(tree.root.join("requirements.md"), AFTER).unwrap();
        tree
    }

    fn run(&self, args: &[&str], code: i32) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert_eq!(
            output.status.code(),
            Some(code),
            "grund {args:?}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// §FS-section-lock.checks.section-address-changed: the 0.20 message, remedy
/// and all, is a verbatim prefix, and what follows it names the citing line.
#[test]
#[ignore = "red until #1022 names the citing sites after the remedy; implement removes this"]
fn the_moved_email_check_message_keeps_its_bytes_as_a_prefix() {
    let tree = Tree::moved();
    let message = tree.run(&["check", "--only", "section-address-changed"], 1);
    assert!(
        message.starts_with(&format!("{FINDING}{REMEDY}")),
        "the 0.20 message is no longer a prefix of:\n{message}"
    );
    assert_eq!(message, format!("{FINDING}{REMEDY}{CITED}\n"));
}

/// §FS-config-explain.4: the answer gives the code's *Fix* and *Note*, and the
/// fix, its slots filled from the finding, is the remedy segment of the check
/// message, so the two agree by construction rather than by matching words
/// (§FS-errors.5.5.1).
#[test]
#[ignore = "red until #1022 names the citing sites after the remedy; implement removes this"]
fn explain_gives_the_moved_email_errors_own_remedy_and_the_lock_note() {
    let tree = Tree::moved();
    let answer = tree.run(&["config", "explain", "section-address-changed"], 0);
    let lines: Vec<&str> = answer.lines().collect();
    assert!(lines.contains(&FIX), "no exact Fix line in:\n{answer}");
    assert!(lines.contains(&NOTE), "no exact Note line in:\n{answer}");
    let filled = FIX
        .trim_start_matches("Fix: ")
        .replace("{new address}", "FS-login.input.1")
        .replace("{cited address}", "FS-login.format.1");
    let message = tree.run(&["check", "--only", "section-address-changed"], 1);
    assert_eq!(message, format!("{FINDING}{filled}{CITED}\n"));
}
