use std::fs;
use std::process::{Command, Output};

fn run(args: &[&str], path: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .arg(path)
        .output()
        .expect("run grund cover")
}

/// §FS-cover.1.1: the shipped CLI rejects an unsupported format before it
/// loads a path whose scan would fail, with the stable usage-error status and
/// diagnostic.
#[test]
fn invalid_cover_format_precedes_missing_path_load() {
    let missing = std::env::temp_dir().join(format!(
        "grund-cover-format-precedence-missing-{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&missing);
    let _ = fs::remove_dir_all(&missing);
    assert!(!missing.exists(), "fixture path must be absent: {missing:?}");

    let invalid = run(&["cover", "--format=bogus"], &missing);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&invalid.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&invalid.stderr),
        "error: unsupported cover format `bogus`\n"
    );

    let valid = run(&["cover", "--format=json"], &missing);
    assert_eq!(valid.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&valid.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&valid.stderr),
        format!("error: path does not exist: {}\n", missing.display())
    );
}
