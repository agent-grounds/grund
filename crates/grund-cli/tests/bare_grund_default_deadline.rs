//! The pending half of the bare-`grund` deprecation ramp: the fallback that
//! still runs `grund check .` with no arguments is removed in grund 0.16.0
//! (§FS-cli.1), which is the named window §REQ-backwards-compatibility.2's
//! deprecation path owes — and bare `grund` is that path's worked example.
//!
//! §FS-distribution.4.2.1: a test can hold only the pending half of a ramp, so
//! this file holds the promise and the release guard holds the other side. The
//! version-gated shape is `unmarked_heading_contract.rs`'s, copied rather than
//! reinvented, because every live ramp in this tree is pinned the same way.

fn version(text: &str) -> Vec<u32> {
    text.trim_end_matches("-dev")
        .split('.')
        .map(|part| part.parse::<u32>().expect("numeric version"))
        .collect()
}

/// §FS-cli.1: the warning bare `grund` prints names grund 0.16.0 as the release
/// the fallback stops running `check .` in, so the deadline it promises is held
/// ahead of the running version — the bump that reaches 0.16.0 fails here rather
/// than shipping a binary whose own message says the fallback is already gone.
#[test]
fn bare_grund_fallback_deadline_is_ahead_of_the_running_version() {
    assert!(
        version(env!("CARGO_PKG_VERSION")) < version("0.16.0"),
        "this tree reached 0.16.0; drop the no-argument `check .` fallback and \
         its warning instead of shipping the warning past its own deadline"
    );
}
