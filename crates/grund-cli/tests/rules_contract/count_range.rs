//! §FS-rule-refusals.sentences.count-range: a count above the largest supported count is refused
//! for its range, naming that count, on both rule surfaces; the largest count itself is accepted.
//! The fixture configures `GOAL`, `REQ`, `FS`, `AR` and `RULE`, and `FS-demo` holds a named
//! `requirements` chapter that cites no `REQ`.

use super::support::{assert_run, fixture, run, scratch, text, unconfigured_rules, write};

/// The refusal a count above the largest supported count is answered with.
fn range_reason() -> String {
    format!(
        "count must be a canonical positive base-10 integer; \
         the count is above the largest supported count, {}",
        usize::MAX
    )
}

/// The first count above the largest supported count.
fn above() -> String {
    (usize::MAX as u128 + 1).to_string()
}

/// The sentences that carry `count`: the three bounds of a relation count and
/// the three of a per-target count.
fn every_count_shape(count: &str) -> Vec<String> {
    ["at least", "at most", "exactly"]
        .into_iter()
        .flat_map(|bound| {
            [
                format!("Each FS must cite {bound} {count} REQ."),
                format!("FS-demo.requirements must cite each REQ {bound} {count} times."),
            ]
        })
        .collect()
}

/// §FS-rule-refusals.sentences.count-range.shapes, §FS-rule-refusals.sentences.count-range.leading-zeros,
/// §FS-rule-refusals.sentences.count-range.no-form: `check --rule` refuses the first count above
/// the largest supported count for its range in every count shape, with or without leading
/// zeros, and prints the reason alone.
#[test]
fn check_rule_refuses_a_count_above_the_largest_supported_count_for_its_range() {
    let root = unconfigured_rules("count-range-rule");
    let expected = format!("error: {}\n", range_reason());
    let wrong: Vec<String> = [above(), format!("0{}", above())]
        .iter()
        .flat_map(|count| every_count_shape(count))
        .filter_map(|sentence| {
            let output = run(&root, &["check", ".", "--rule", &sentence]);
            let (exit, stderr) = (output.status.code(), text(&output.stderr));
            (exit != Some(2) || stderr != expected).then(|| {
                format!("--rule {sentence:?}: exit {exit:?}\n  stderr   {stderr:?}\n  expected {expected:?}")
            })
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rule-refusals.sentences.count-range.shapes: a configured rule declaration whose count is above
/// the largest supported count carries the same reason in its `invalid-rule` finding.
#[test]
fn a_configured_rule_with_a_count_above_the_largest_supported_count_names_the_range() {
    let root = scratch("count-range-configured");
    let sentences = every_count_shape(&above());
    for (index, sentence) in sentences.iter().enumerate() {
        write(
            &root,
            &format!("docs/rules/RULE-range-{index}.md"),
            &format!("# RULE-range-{index}: {sentence}\n\nBecause \u{a7}GOAL-rules.\n"),
        );
    }
    let stdout = text(&run(&root, &["check", "."]).stdout);
    let wrong: Vec<String> = (0..sentences.len())
        .filter_map(|index| {
            let id = format!("RULE-range-{index}");
            let expected = format!(
                "docs/rules/{id}.md:1: error: {id} is not a valid rule: {}",
                range_reason()
            );
            let actual = stdout
                .lines()
                .find(|line| line.contains(&format!(" {id} is not a valid rule: ")))
                .unwrap_or("no invalid-rule finding");
            (actual != expected)
                .then(|| format!("{id}\n  finding  {actual:?}\n  expected {expected:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Guard for §FS-rules.3, green before and after §FS-rule-refusals.sentences.count-range: the
/// largest supported count reaches rule evaluation in every count shape, the issue's control.
#[test]
fn the_largest_supported_count_is_accepted_in_every_count_shape() {
    let root = unconfigured_rules("count-range-largest");
    for sentence in every_count_shape(&usize::MAX.to_string()) {
        let output = run(&root, &["check", ".", "--rule", &sentence]);
        let stderr = text(&output.stderr);
        assert!(
            matches!(output.status.code(), Some(0 | 1)) && !stderr.contains("error: count"),
            "--rule {sentence:?} was refused: exit {:?}\n{stderr}",
            output.status.code()
        );
    }
}

/// Guard for §FS-rule-refusals.sentences.rewrite.by-refusal, green before and after: a count in
/// range with leading zeros keeps its form, and zero and a word keep their refusal.
#[test]
fn the_neighbouring_count_refusals_are_unchanged() {
    let largest = format!("Each FS must cite at most {} REQ.", usize::MAX);
    let canonical = "count must be a canonical positive base-10 integer";
    for (sentence, refusal) in [
        (
            "Each FS must cite at most 007 REQ.".to_string(),
            format!("{canonical}; accepted form: Each FS must cite at most 7 REQ."),
        ),
        (
            largest.replace("at most ", "at most 0"),
            format!("{canonical}; accepted form: {largest}"),
        ),
        ("Each FS must cite at least 0 REQ.".into(), canonical.into()),
        ("Each FS must cite exactly 00 REQ.".into(), canonical.into()),
        (
            "Each FS must cite at least two REQ.".into(),
            canonical.into(),
        ),
    ] {
        let output = run(&fixture(), &["check", ".", "--rule", &sentence]);
        assert_run(&output, 2, "", &format!("error: {refusal}\n"));
    }
}
