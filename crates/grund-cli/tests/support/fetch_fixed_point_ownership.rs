//! Changed refreshes and ordered insertion preserve every sibling's lead and the existing
//! separators, including after an originally unterminated prefix (§FS-fetch.6.1).

use super::{AFTER, Home, MID, Project, same};

fn preamble(eol: &str) -> &'static str {
    if eol == "\r\n" {
        "# Architecture\r\n\r\n"
    } else {
        "# Architecture\n\n"
    }
}

fn at_line_start(home: &str, output: &str) -> Result<(), String> {
    if home
        .match_indices(output)
        .any(|(at, _)| at == 0 || home.as_bytes()[at - 1] == b'\n')
    {
        Ok(())
    } else {
        Err(format!(
            "output {output:?} is not contiguous at a line start in {home:?}"
        ))
    }
}

fn unchanged(p: &Project, ids: &[&str], expected: &str) -> Result<(), String> {
    let docs = p.docs();
    for id in ids {
        p.fetch_ok(id)?;
        same(&format!("unchanged {id}"), expected, &p.arch())?;
        if p.docs() != docs {
            return Err(format!("unchanged {id} changed the docs tree"));
        }
    }
    p.check_green()
}

fn changed_before_sibling(eol: &str, lead: &str, trailing: &str) -> Result<(), String> {
    let p = Project::new(Home::File(Some(preamble(eol))));
    let old = format!("## {MID}: Mid{eol}old{eol}{trailing}");
    let sibling = format!("{lead}## {AFTER}: After{eol}kept{eol}");
    p.print(MID, &old);
    p.print(AFTER, &sibling);
    p.fetch_ok(MID)?;
    p.fetch_ok(AFTER)?;
    let separator = if trailing.is_empty() { eol } else { "" };
    let before = format!("{}{old}{separator}{sibling}", preamble(eol));
    same("after fetching both IDs", &before, &p.arch())?;
    at_line_start(&before, &sibling)?;

    let expected = before.replace(&format!("{eol}old{eol}"), &format!("{eol}new{eol}"));
    p.print(MID, &old.replace("old", "new"));
    p.fetch_ok(MID)?;
    same("changed A preserves all of B's lead", &expected, &p.arch())?;
    at_line_start(&p.arch(), &sibling)?;
    unchanged(&p, &[AFTER, MID], &expected)
}

/// R1-01: real replacements keep B's entire lead, including beside A's own trailing lines.
#[test]
fn fetch_changed_output_preserves_sibling_lead() {
    let mut failures = Vec::new();
    for eol in ["\n", "\r\n"] {
        let leads = [
            ("no-lead", String::new()),
            ("two-blank-lines", eol.repeat(2)),
            ("space-only-line", format!(" {eol}")),
            ("nbsp-only-line", format!("\u{a0}{eol}")),
        ];
        for (name, lead) in leads {
            for trailing_lines in [0, 2] {
                if let Err(err) = changed_before_sibling(eol, &lead, &eol.repeat(trailing_lines)) {
                    failures.push(format!("{eol:?}/{name}/trailing-{trailing_lines}: {err}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn insert_before_sibling(eol: &str, lead: &str) -> Result<(), String> {
    let p = Project::new(Home::File(Some(preamble(eol))));
    let inserted = format!("## {MID}: Mid{eol}old{eol}");
    let sibling = format!("{lead}## {AFTER}: After{eol}kept{eol}");
    p.print(MID, &inserted);
    p.print(AFTER, &sibling);
    p.fetch_ok(AFTER)?;
    same(
        "B fetched first",
        &format!("{}{sibling}", preamble(eol)),
        &p.arch(),
    )?;
    p.fetch_ok(MID)?;
    let expected = format!("{}{inserted}{eol}{sibling}", preamble(eol));
    same("A inserted ahead of B's lead", &expected, &p.arch())?;
    at_line_start(&p.arch(), &sibling)?;
    unchanged(&p, &[AFTER, MID], &expected)
}

/// R1-02: B fetched first keeps its complete output after insertion of an earlier ID.
#[test]
fn fetch_ordered_insertion_preserves_sibling_lead() {
    let mut failures = Vec::new();
    for eol in ["\n", "\r\n"] {
        for (name, lead) in [
            ("no-lead", String::new()),
            ("two-blank-lines", eol.repeat(2)),
            ("space-only-line", format!(" {eol}")),
            ("nbsp-only-line", format!("\u{a0}{eol}")),
        ] {
            if let Err(err) = insert_before_sibling(eol, &lead) {
                failures.push(format!("{eol:?}/{name}: {err}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn changed_after_unterminated_prefix(sibling: bool) -> Result<(), String> {
    let p = Project::new(Home::File(Some(if sibling {
        "# Architecture\n\n"
    } else {
        "# Architecture"
    })));
    const BEFORE: &str = "AR-001-before";
    if sibling {
        p.print(BEFORE, "## AR-001-before: Before\nkept");
        p.fetch_ok(BEFORE)?;
    }
    let prefix = p.arch();
    let old = format!("\n## {MID}: Mid\nold\n");
    p.print(MID, &old);
    p.fetch_ok(MID)?;
    let first = format!("{prefix}\n{old}");
    same(
        "first insertion after unterminated prefix",
        &first,
        &p.arch(),
    )?;
    for body in ["new", "latest"] {
        p.print(MID, &old.replace("old", body));
        p.fetch_ok(MID)?;
        let expected = first.replace("\nold\n", &format!("\n{body}\n"));
        same(
            "changed text keeps the left separator",
            &expected,
            &p.arch(),
        )?;
        unchanged(&p, &[MID], &expected)?;
        if sibling {
            unchanged(&p, &[BEFORE], &expected)?;
        }
    }
    Ok(())
}

/// R1-03: two successive body changes reuse the separator installed after an unterminated
/// preamble or snapshot, rather than adding padding on every real replacement.
#[test]
fn fetch_changed_output_preserves_left_separator() {
    let failures = [false, true]
        .into_iter()
        .filter_map(|sibling| {
            changed_after_unterminated_prefix(sibling)
                .err()
                .map(|err| format!("preceding-sibling={sibling}: {err}"))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
