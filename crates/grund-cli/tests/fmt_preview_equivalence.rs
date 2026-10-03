//! §FS-fmt.7.3 — the line set `fmt --check` prints is the change set `fmt
//! --write` applies. The dry run is the write run with the write withheld, not
//! a second computation of the same question.
//!
//! The preview is read from the report; the change set is read from the **disk**
//! rather than from the write run's own summary, because a summary is the
//! second computation agreeing with itself. Two fresh copies of one shape are
//! built, one previewed and one written, and the lines that actually differ
//! after the write are compared to the lines the preview named.
//!
//! Unix only: the corpus needs real broken symlinks.

#![cfg(unix)]

#[path = "support/fmt_shapes.rs"]
mod fmt_shapes;

use fmt_shapes::{changed_sites, reported_sites, run_grund, shapes, snapshot, stderr, stdout};

/// The declaration-local sites §FS-fmt.7.5.2 requires the rewritable shape to
/// hold: one whose target section exists and one whose target does not
/// (§FS-fmt.2.4.6). Written with `\u{a7}` because `$$2.1` is not a candidate.
const RESOLVING_LOCAL_SITE: &str = "\u{a7}2.1";
const ABSENT_LOCAL_SITE: &str = "\u{a7}12";

/// The rewrite knobs, each previewed and applied. The bare form is the one every
/// gate runs; `--cross-refs` and `--marker` add a rewrite class each, and a
/// preview that predicts one class and not another is the defect this rule is
/// about.
const KNOBS: &[&[&str]] = &[
    &[],
    &["--cross-refs"],
    &["--marker"],
    &["--marker", "--cross-refs"],
];

#[test]
fn the_dry_run_names_exactly_the_lines_the_write_changes() {
    for shape in shapes() {
        for knob in KNOBS {
            let previewed = shape.materialize(&format!("preview-check{}", knob.join("-")));
            let written = shape.materialize(&format!("preview-write{}", knob.join("-")));
            let before = snapshot(&written);

            let mut check = vec!["fmt", "--check"];
            check.extend_from_slice(knob);
            let mut write = vec!["fmt", "--write"];
            write.extend_from_slice(knob);

            let preview = run_grund(&check, &previewed);
            let applied = run_grund(&write, &written);

            let form = format!(
                "{} / `grund fmt [--check|--write] {}`",
                shape.name,
                knob.join(" ")
            );
            assert_eq!(
                reported_sites(&preview),
                changed_sites(&before, &snapshot(&written)),
                "{form}: the preview and the write disagree about which lines change\n  \
                 preview stdout: {}\n  write stdout: {}\n  write stderr: {}",
                stdout(&preview),
                stdout(&applied),
                stderr(&applied),
            );
        }
    }
}

#[test]
/// A preview whose lines the write refuses is a finding no edit can clear, so a
/// gate built on `fmt --check` could never go green. Beyond agreeing on *which*
/// lines change, the two modes must agree on whether there is anything to do —
/// which is the exit code, and the exit code is what a gate reads. An
/// unreadable path outranks both verdicts and is reported as `2` by each mode
/// alike (§FS-fmt.3), so it is the one case where `1` says nothing.
fn the_two_modes_agree_on_whether_the_tree_is_clean() {
    for shape in shapes() {
        let previewed = shape.materialize("preview-clean-check");
        let written = shape.materialize("preview-clean-write");
        let before = snapshot(&written);

        let preview = run_grund(&["fmt", "--check"], &previewed);
        let applied = run_grund(&["fmt", "--write"], &written);
        let changed = changed_sites(&before, &snapshot(&written));

        assert_ne!(
            applied.status.code(),
            Some(1),
            "{}: `--write` returned the dry run's exit code (§FS-fmt.3)",
            shape.name,
        );
        assert_eq!(
            preview.status.code() == Some(2),
            applied.status.code() == Some(2),
            "{}: one mode met an unreadable path and the other did not\n  \
             check stderr: {}\n  write stderr: {}",
            shape.name,
            stderr(&preview),
            stderr(&applied),
        );
        if preview.status.code() == Some(2) {
            continue;
        }
        assert_eq!(
            preview.status.code() == Some(1),
            !changed.is_empty(),
            "{}: `--check` exited {:?} while the write changed {} line(s)",
            shape.name,
            preview.status.code(),
            changed.len(),
        );
    }
}

#[test]
/// The other half of the corpus contract: a shape declared to rewrite must
/// still rewrite. A preview that predicts nothing matches a write that does
/// nothing, so §FS-fmt.7.3 needs shapes where both sides are non-empty.
fn every_shape_rewrites_exactly_where_it_claims_to() {
    for shape in shapes() {
        let root = shape.materialize("preview-coverage");
        let before = snapshot(&root);
        let output = run_grund(&["fmt", "--write"], &root);
        let changed = changed_sites(&before, &snapshot(&root));
        assert_eq!(
            !changed.is_empty(),
            shape.rewrites,
            "{}: the shape's `rewrites` no longer describes it: {}{}",
            shape.name,
            stdout(&output),
            stderr(&output),
        );
    }
}

#[test]
/// §FS-fmt.7.5.2 — the declaration-local class asserted by its own lines, not by
/// the shape: `clean-markdown` also rewrites through its notes file, so the
/// shape-level assertions stay green if this class stops firing. The site whose
/// target exists is previewed and written; the one whose target is absent is
/// neither (§FS-fmt.2.4.6), each on its own line so a split cannot hide.
fn the_declaration_local_rewrite_is_previewed_and_written_on_its_own_lines() {
    let shape = shapes()
        .into_iter()
        .find(|shape| shape.name == "clean-markdown")
        .expect("the corpus holds the clean rewritable shape");
    let previewed = shape.materialize("preview-local-check");
    let written = shape.materialize("preview-local-write");
    let before = snapshot(&written);

    let preview = run_grund(&["fmt", "--check"], &previewed);
    run_grund(&["fmt", "--write"], &written);
    let listed = reported_sites(&preview);
    let changed = changed_sites(&before, &snapshot(&written));

    let site_of = |token: &str| {
        let sites: Vec<(String, usize)> = before
            .iter()
            .flat_map(|(path, lines)| {
                lines
                    .iter()
                    .enumerate()
                    .filter(|(_, line)| {
                        line.match_indices(token).any(|(at, _)| {
                            !line[at + token.len()..]
                                .starts_with(|c: char| c.is_ascii_digit() || c == '.')
                        })
                    })
                    .map(move |(index, _)| (path.clone(), index + 1))
            })
            .collect();
        assert_eq!(
            sites.len(),
            1,
            "clean-markdown must hold `{token}` on exactly one line (§FS-fmt.7.5.2), found {sites:?}",
        );
        sites.into_iter().next().unwrap()
    };
    let resolving = site_of(RESOLVING_LOCAL_SITE);
    let absent = site_of(ABSENT_LOCAL_SITE);
    assert_ne!(
        resolving, absent,
        "the two declaration-local sites share a line"
    );

    assert!(
        changed.contains(&resolving) && listed.contains(&resolving),
        "{resolving:?} cites a section the declaration records, so both modes rewrite it\n  \
         listed: {listed:?}\n  changed: {changed:?}\n  preview stdout: {}",
        stdout(&preview),
    );
    assert!(
        !changed.contains(&absent) && !listed.contains(&absent),
        "{absent:?} cites a section the declaration does not record, so neither mode \
         rewrites it (§FS-fmt.2.4.6)\n  listed: {listed:?}\n  changed: {changed:?}",
    );
}
