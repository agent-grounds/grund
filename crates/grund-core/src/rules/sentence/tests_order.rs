//! §FS-rule-refusals.sentences.order: the stages `read` takes, held to the
//! specification's table so the two cannot drift.

use std::path::Path;

use super::STAGES;

#[test]
fn read_takes_the_stages_of_the_specification_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/functional-spec/FS-rule-refusals.md");
    let spec = std::fs::read_to_string(&path).expect("read FS-rule-refusals.md");
    let table = spec
        .split("### sentences.order:")
        .nth(1)
        .expect("the sentences.order point");
    let stages = table
        .lines()
        .filter_map(|line| line.strip_prefix("| "))
        .filter_map(|row| {
            let (number, rest) = row.split_once(". ")?;
            number.parse::<usize>().ok()?;
            Some(rest.split(" |").next()?.trim())
        })
        .collect::<Vec<_>>();
    assert_eq!(stages, STAGES);
}
