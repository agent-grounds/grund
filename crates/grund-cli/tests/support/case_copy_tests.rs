//! §FS-examples.5.6.2: only owned paths change at the shared output seam.
use super::{copy_path_spellings, normalize_copy_output};

#[test]
fn complete_path_fields_preserve_sibling_and_enclosing_names() {
    let actual = "target/e2e-work/invocation-sample/repo";
    let logical = "target/e2e-work/example/repo";
    let prefixes = vec![(actual.into(), logical.into())];
    let input = format!(
        "{actual}\n'{actual}' \"{actual}/nested file\" `{actual}/child`\n\
         {{\"path\":\"{actual}/child\"}}\n{actual}/file.md:12: error\n\
         grund's copy: {actual}\n"
    );
    assert_eq!(
        normalize_copy_output(&input, &prefixes, false),
        input.replace(actual, logical)
    );
    // These are different path fields, including punctuation beyond @ and +.
    // A quoted field can contain filename spaces without becoming two paths.
    for other in [
        format!("{actual}@other"),
        format!("{actual}+other"),
        format!("{actual},other"),
        format!("{actual}(other)"),
        format!("{actual}:other"),
        format!("{actual}💾other"),
        format!("prefix@{actual}"),
        format!("prefix+{actual}"),
        format!("/{actual}"),
        format!("'{actual} other'"),
        format!("\"prefix@{actual}\""),
        format!("literal\\{actual}"),
        format!("{actual}\\unix-filename"),
        format!("'{actual}'@other"),
        format!("prefix+'{actual}'"),
        format!("{actual}'other'"),
        format!("{actual}\"other\""),
        format!("{actual}`other`"),
    ] {
        assert_eq!(normalize_copy_output(&other, &prefixes, false), other);
    }
    let absolute = format!("/root with spaces/{actual}");
    let stable = format!("/root with spaces/{logical}");
    assert_eq!(
        normalize_copy_output(
            &format!("copy: '{absolute}/nested file'\nabsolute: {absolute}\n"),
            &[(absolute, stable.clone())],
            false
        ),
        format!("copy: '{stable}/nested file'\nabsolute: {stable}\n")
    );
}

/// Portable spelling evidence, not a claim of native Windows execution.
#[test]
fn windows_native_relative_output_keeps_the_authored_logical_spelling() {
    let actual = r"target/e2e-work\invocation-sample\repo";
    let logical = r"target/e2e-work\example\repo";
    let prefixes = copy_path_spellings(actual, logical, true);
    for emitted in [actual, r"target\e2e-work\invocation-sample\repo"] {
        assert_eq!(normalize_copy_output(emitted, &prefixes, true), logical);
        assert_eq!(
            normalize_copy_output(&format!("'{emitted}\\child'"), &prefixes, true),
            format!("'{logical}\\child'")
        );
        for other in [format!("{emitted}@other"), format!("literal\\{emitted}")] {
            assert_eq!(normalize_copy_output(&other, &prefixes, true), other);
        }
    }
    assert_eq!(
        normalize_copy_output(
            "target/e2e-work/invocation-sample/repo/child",
            &prefixes,
            true
        ),
        "target/e2e-work/example/repo/child"
    );
    let absolute = copy_path_spellings(
        r"\\?\C:\work\target/e2e-work\invocation-sample\repo",
        r"\\?\C:\work\target/e2e-work\example\repo",
        true,
    );
    assert_eq!(
        normalize_copy_output(
            r"C:\work\target\e2e-work\invocation-sample\repo",
            &absolute,
            true
        ),
        r"C:\work\target/e2e-work\example\repo"
    );
    // On Unix a backslash is a filename byte: no alternate path is registered.
    let unix = copy_path_spellings("target/invocation/repo", "target/example/repo", false);
    assert_eq!(
        normalize_copy_output(r"target\invocation\repo", &unix, false),
        r"target\invocation\repo"
    );
}

#[path = "case_copy_output_probe.rs"]
mod output_probe;
