//! §FS-scan.path.warning.release: the release every narrowing line names is one
//! constant, and the release guard's vocabulary does not read it, so this test is
//! what holds it ahead of the crate (§FS-distribution.4.2.1).

use super::narrowing::PATH_RESOLVES_PROJECT_WIDE;

#[test]
fn the_path_narrowing_release_is_still_ahead() {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse().expect("numeric version"))
            .collect()
    };
    assert!(
        parse(env!("CARGO_PKG_VERSION")) < parse(PATH_RESOLVES_PROJECT_WIDE),
        "grund {} has reached {PATH_RESOLVES_PROJECT_WIDE}: a path must now narrow only what \
         is reported (§DF-path-narrowing-warns), and the warnings go",
        env!("CARGO_PKG_VERSION")
    );
}
