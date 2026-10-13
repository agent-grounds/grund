//! Test module: the release `parse-moves` names is still ahead
//! (§FS-declarations.checks.parse-moves.4).

use super::parse_moves::ONE_PARSE_FLIP;

/// §FS-declarations.checks.parse-moves.4: every message names a release still to come; once
/// this crate reaches it, the answers have moved and the check must go, and this test says so.
#[test]
fn the_one_parse_flip_is_still_ahead() {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse().expect("numeric version"))
            .collect()
    };
    assert!(
        parse(env!("CARGO_PKG_VERSION")) < parse(ONE_PARSE_FLIP),
        "grund {} has reached {ONE_PARSE_FLIP}: remove parse-moves",
        env!("CARGO_PKG_VERSION")
    );
}
