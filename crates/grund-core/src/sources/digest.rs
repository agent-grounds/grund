//! grund's one SHA-256 (§FS-section-lock.holds.fingerprint): the digest
//! §FS-remote-projects.bytes.digest names for a projection's exact bytes, and the
//! one a section lock's fingerprint is cut from. It sits with the bytes rather than
//! with either reader, so the two cannot come to disagree on the algorithm.

use sha2::{Digest, Sha256};

/// The SHA-256 of `bytes`, as 64 lowercase hex digits.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn the_empty_input_has_the_published_digest() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
