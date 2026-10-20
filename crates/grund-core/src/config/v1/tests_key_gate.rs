//! §AR-config.3.3 item 5: `KEYS` is the v1 reader's outer-key gate. A key it
//! does not name is refused with the reader's `unknown config key` text, and the
//! refusals that word themselves keep their words, because they run first; so
//! does the `lead_size_warning` member gate (§AR-config.9.1).

use std::path::Path;

use super::mapping::{KEYS, admits};
use super::read_sections;

fn refusal(text: &str) -> String {
    match read_sections(text, Path::new("grund.toml")) {
        Ok(_) => panic!("the file reads:\n{text}"),
        Err(error) => error.to_string(),
    }
}

/// Every row of `KEYS` is admitted where the reader holds it, a per-kind
/// citation table under any kind's name.
#[test]
fn the_gate_admits_every_row_of_the_table() {
    for (name, _) in KEYS {
        let (section, key) = match name.split_once("] ") {
            Some((header, key)) => (header.trim_matches(['[', ']']).replace("<KIND>", "FS"), key),
            None => (String::new(), *name),
        };
        assert!(admits(&section, key), "`{name}` is not admitted");
    }
    assert!(!admits("reference", "kind"));
    assert!(!admits("", "marker"));
    assert!(!admits("citations.FS", "fetch"));
}

/// §FS-config.4.3: a key outside the table is refused at its line, in the
/// reader's words, in every section that has no words of its own.
#[test]
fn a_key_outside_the_table_is_an_unknown_config_key() {
    for (text, line) in [
        ("marker = \"#\"\n", 1),
        ("[reference]\nkind = \"FS\"\n", 2),
        ("[[kinds]]\nkind = \"FS\"\nfolders = [\"docs\"]\n", 3),
        ("[workspace]\nmember = [\"a\"]\n", 2),
    ] {
        let key = text
            .lines()
            .nth(line - 1)
            .unwrap()
            .split(" = ")
            .next()
            .unwrap();
        assert_eq!(
            refusal(text),
            format!("grund.toml:{line}: unknown config key `{key}`")
        );
    }
}

/// §FS-config.3.4.6, §FS-config.3.9, §FS-config.3.1.2: the refusals that
/// word themselves still do, ahead of the gate.
#[test]
fn the_refusals_with_their_own_words_keep_them() {
    for (text, words) in [
        (
            "[[kinds]]\nprefix = \"FS\"\n",
            "grund.toml:2: [[kinds]] `prefix` was removed in grund 0.13.0 — rename it to `kind`",
        ),
        (
            "[citations]\nmust = [\"FS\"]\n",
            "grund.toml:2: unknown key `must` in [citations] (expected `default`, or a \
             [citations.<KIND>] table)",
        ),
        (
            "[citations.FS]\nfetch = \"x\"\n",
            "grund.toml:2: unknown key `fetch` in [citations.FS] (expected must, should, may, \
             should-not, must-not, or default)",
        ),
        (
            "[reference]\nlead_size_warning = { max = 3, units = \"words\" }\n",
            "grund.toml:2: unknown lead_size_warning field `units`",
        ),
        (
            "[reference]\nlead_size_warning = { max = 3, = 1 }\n",
            "grund.toml:2: invalid lead_size_warning field",
        ),
    ] {
        assert_eq!(refusal(text), words);
    }
}
