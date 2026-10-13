//! §FS-cli.2.3.1: every flag a subcommand's help page names has a one-line example —
//! a command line running that subcommand with the flag, on the flag's own entry after
//! `e.g.` or in the page's examples block. The sweep reads the subcommands off the
//! top-level page, so a page added later is held to the same rule.

use std::collections::BTreeSet;
use std::process::Command;

fn help(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .output()
        .expect("run grund");
    assert!(
        out.status.success(),
        "grund {args:?} exited {:?}",
        out.status
    );
    String::from_utf8(out.stdout).expect("utf-8 help")
}

/// The subcommands the top-level page lists: the first word of each indented row
/// under a group heading (§FS-cli.2.2).
fn subcommands() -> Vec<String> {
    let page = help(&["help"]);
    let mut in_group = false;
    let mut names = Vec::new();
    for line in page.lines() {
        if line.is_empty() {
            in_group = false;
        } else if !line.starts_with(' ') && line.ends_with(':') {
            in_group = true;
        } else if in_group {
            if let Some(name) = line
                .strip_prefix("  ")
                .and_then(|l| l.split_whitespace().next())
            {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The `--name` of every `--flag` token in `text`, cut at `[`, `=` and the like.
fn flag_tokens(text: &str) -> Vec<String> {
    let mut flags = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("--") {
        let before = rest[..at].chars().next_back();
        let tail = &rest[at + 2..];
        let len = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(tail.len());
        if !matches!(before, Some(c) if c.is_ascii_alphanumeric() || c == '-') && len > 0 {
            flags.push(format!("--{}", &tail[..len]));
        }
        rest = &tail[len..];
    }
    flags
}

/// Whether `text` holds a command line running `cmd` that passes `flag`.
fn invokes(text: &str, cmd: &str, flag: &str) -> bool {
    let mut prefixes = vec![format!("grund {cmd} ")];
    if cmd == "show" {
        prefixes.push("grund ".to_string());
    }
    prefixes.iter().any(|prefix| {
        text.match_indices(prefix.as_str()).any(|(at, _)| {
            let line = &text[at..];
            let line = line.split(" # ").next().unwrap_or(line);
            flag_tokens(line).iter().any(|f| f == flag)
        })
    })
}

/// The flags `page` names (usage lines and option entries) that have no example.
fn missing_examples(cmd: &str, page: &str) -> Vec<String> {
    let lines: Vec<&str> = page.lines().collect();
    let mut named = BTreeSet::new();
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut examples = String::new();
    let (mut in_usage, mut in_examples) = (false, false);
    for line in &lines {
        let trimmed = line.trim_start();
        if line.is_empty() {
            in_usage = false;
            in_examples = false;
            continue;
        }
        if line.starts_with("Usage:") {
            in_usage = true;
        } else if !line.starts_with(' ') {
            in_usage = false;
            in_examples = line.to_lowercase().ends_with("examples:");
            continue;
        }
        if in_usage {
            named.extend(flag_tokens(line));
        } else if in_examples {
            examples.push_str(line);
            examples.push('\n');
        } else if line.starts_with("  --") {
            let flag = flag_tokens(trimmed).into_iter().next().expect("entry flag");
            named.insert(flag.clone());
            entries.push((flag, (*line).to_string()));
        } else if line.starts_with("   ") && !trimmed.starts_with("--") {
            if let Some((_, entry)) = entries.last_mut() {
                entry.push('\n');
                entry.push_str(line);
            }
        }
    }
    named
        .into_iter()
        .filter(|flag| {
            let inline = entries.iter().filter(|(f, _)| f == flag).any(|(_, entry)| {
                entry
                    .split("e.g. ")
                    .skip(1)
                    .any(|after| invokes(after, cmd, flag))
            });
            !inline && !invokes(&examples, cmd, flag)
        })
        .collect()
}

#[test]
fn every_flag_on_every_subcommand_page_has_an_example() {
    let commands = subcommands();
    assert!(
        commands.len() >= 10,
        "top-level page lists too few commands: {commands:?}"
    );
    let mut missing = Vec::new();
    for cmd in &commands {
        let page = help(&[cmd.as_str(), "--help"]);
        for flag in missing_examples(cmd, &page) {
            missing.push(format!("{cmd} {flag}"));
        }
    }
    assert!(
        missing.is_empty(),
        "{} flag(s) have no one-line example (§FS-cli.2.3.1):\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

/// The rule itself, on pages written to the edge of it.
#[test]
fn an_example_is_an_invocation_on_the_entry_or_in_the_examples_block() {
    let page = "Usage:  grund list [--kind K] [--format F] [--top N]\n\nOptions:\n\
        \x20 --kind K   only kinds  e.g. grund list --kind FS\n\
        \x20 --selector S  a selector, e.g. FS.requirements\n\
        \x20 --format F  the format\n\
        \x20 --top N    largest N\n\
        \x20            e.g. grund refs X --top 3\n\n\
        Examples:\n  grund list --format=json   # json\n";
    assert_eq!(missing_examples("list", page), ["--selector", "--top"]);
    let show = "Usage:  grund [show] <ID> [--toc] [--path P]\n\nExamples:\n  grund FS-x --toc\n";
    assert_eq!(missing_examples("show", show), ["--path"]);
}
