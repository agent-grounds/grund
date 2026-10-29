//! The fingerprint a section lock records and `check` compares
//! (§FS-section-lock.holds.fingerprint): the digest of the text
//! `relative_fingerprint` gives a locked section, which `section_lock` computes
//! for every section of a cited declaration in one place
//! (§DF-section-lock-relative-fingerprint.decision.one-place).

use crate::sources::sha256_hex;

/// The fingerprint of a section's text (§FS-section-lock.holds.fingerprint): the
/// first sixteen lowercase hex digits of the SHA-256 of `text` with LF line
/// endings, each line's trailing spaces and tabs removed, leading and trailing
/// empty lines dropped, and the lines joined with LF and no final LF.
pub(super) fn section_fingerprint(text: &str) -> String {
    let lines = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>();
    let first = lines.iter().position(|line| !line.is_empty());
    let last = lines.iter().rposition(|line| !line.is_empty());
    let normalized = match (first, last) {
        (Some(first), Some(last)) => lines[first..=last].join("\n"),
        _ => String::new(),
    };
    sha256_hex(normalized.as_bytes())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::section_fingerprint;

    #[test]
    fn the_fingerprint_is_the_digest_of_the_normalized_text() {
        assert_eq!(
            section_fingerprint("Trim surrounding whitespace."),
            "8f332b6fe87bb346"
        );
        assert_eq!(section_fingerprint("Split on commas."), "be879e46c73e3d05");
    }

    #[test]
    fn line_endings_and_trailing_blanks_do_not_change_it() {
        let plain = section_fingerprint("One.\nTwo.");
        assert_eq!(section_fingerprint("\r\nOne.  \r\nTwo.\t\r\n\r\n"), plain);
        assert_eq!(section_fingerprint("\n\nOne.\nTwo.\n"), plain);
        assert_ne!(section_fingerprint("One.\n\nTwo."), plain);
    }
}
