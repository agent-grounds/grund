//! §FS-fmt.6.3, §FS-fmt.requirements.fixed-point — one `fmt --write` is a fixed
//! point: a second run with no intervening edits finds nothing to do. The property
//! compares two runs, so no single golden states it; it is asserted over every
//! shape of the corpus.
//!
//! The shape that makes it more than vacuous is `clean-markdown`, whose section
//! headings carry citations the run expands while other lines link to those
//! headings (§FS-fmt.6.3.3): a link written with the anchor of the heading as it
//! was read is rewritten again by the next run.
//!
//! Unix only: the corpus needs real broken symlinks.

#![cfg(unix)]

#[path = "support/fmt_shapes.rs"]
mod fmt_shapes;

use fmt_shapes::{changed_sites, reported_sites, run_grund, shapes, snapshot, stderr, stdout};

/// The rewrite knobs, as in the preview-equivalence suite: each adds a class,
/// and a class the first write leaves half-done is the defect this rule is about.
const KNOBS: &[&[&str]] = &[
    &[],
    &["--cross-refs"],
    &["--marker"],
    &["--marker", "--cross-refs"],
];

#[test]
fn a_second_write_changes_nothing_and_the_check_after_a_write_is_clean() {
    for shape in shapes() {
        for knob in KNOBS {
            let root = shape.materialize(&format!("fixed-point{}", knob.join("-")));
            let mut write = vec!["fmt", "--write"];
            write.extend_from_slice(knob);
            let mut check = vec!["fmt", "--check"];
            check.extend_from_slice(knob);
            let form = format!("{} / `grund fmt {}`", shape.name, knob.join(" "));

            let first = run_grund(&write, &root);
            let once = snapshot(&root);
            let after = run_grund(&check, &root);
            assert!(
                reported_sites(&after).is_empty() && after.status.code() != Some(1),
                "{form}: `--check` straight after a `--write` still finds work \
                 (§FS-fmt.6.3)\n  first write stdout: {}\n  check exit: {:?}\n  \
                 check stdout: {}\n  check stderr: {}",
                stdout(&first),
                after.status.code(),
                stdout(&after),
                stderr(&after),
            );

            let second = run_grund(&write, &root);
            let changed = changed_sites(&once, &snapshot(&root));
            assert!(
                changed.is_empty(),
                "{form}: a second `--write` rewrote lines the first left behind \
                 (§FS-fmt.6.3)\n  changed: {changed:?}\n  first write stdout: {}\n  \
                 second write stdout: {}",
                stdout(&first),
                stdout(&second),
            );
        }
    }
}

#[test]
/// §FS-fmt.6.3.3 — the heading sites asserted by their own lines, so the
/// property above cannot pass because the class stopped firing: the first write
/// expands both headings and links both notes lines, and it leaves each link
/// with the anchor of the heading as that write left it.
fn a_heading_the_run_expands_is_linked_with_the_anchor_it_leaves() {
    let shape = shapes()
        .into_iter()
        .find(|shape| shape.name == "clean-markdown")
        .expect("the corpus holds the clean rewritable shape");
    let root = shape.materialize("fixed-point-headings");
    let before = snapshot(&root);
    let output = run_grund(&["fmt", "--write"], &root);
    let after = snapshot(&root);
    let changed = changed_sites(&before, &after);

    let line_of = |path: &str, prefix: &str| -> (String, usize, String) {
        let lines = &after[path];
        let index = lines
            .iter()
            .position(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("{path} holds a line starting `{prefix}`: {lines:?}"));
        (path.to_string(), index + 1, lines[index].clone())
    };
    let expected = [
        (
            line_of("docs/FS-002-beta.md", "## 1. "),
            "## 1. See [\u{a7}FS-001-alpha](FS-001-alpha.md#fs-001-alpha-alpha)",
        ),
        (
            line_of("docs/FS-002-beta.md", "## 2. "),
            "## 2. Back to [\u{a7}FS-002-beta.1](FS-002-beta.md#1-see-fs-001-alpha)",
        ),
        (
            line_of("docs/heading-notes.md", "The shorthand heading: "),
            "The shorthand heading: [\u{a7}FS-002-beta.1](FS-002-beta.md#1-see-fs-001-alpha)",
        ),
        (
            line_of("docs/heading-notes.md", "The local heading: "),
            "The local heading: [\u{a7}FS-002-beta.2](FS-002-beta.md#2-back-to-fs-002-beta1)",
        ),
    ];
    for ((path, number, written), want) in expected {
        assert!(
            changed.contains(&(path.clone(), number)),
            "{path}:{number} holds a citation the first write must rewrite\n  stdout: {}",
            stdout(&output),
        );
        assert_eq!(
            written, want,
            "{path}:{number}: the link names the heading as the run read it, not as it \
             left it (§FS-fmt.6.3.3)",
        );
    }
}
