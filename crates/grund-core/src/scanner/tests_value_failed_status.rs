//! A failed status is never a determination that a listed file is no source (§FS-values.5.3.1),
//! even where the rule reads its target as none (§FS-scan.identity.reading.entry). A failed stat
//! also fails the resolution, so only a spelling this run resolved earlier reaches the seam: the
//! disk changed under the run, and the one way a format of `None` meets a failed status.

use std::path::PathBuf;

use super::value_json::scan_value_sources;
use crate::config::load_config;
use crate::model::Catalog;
use crate::testing::{DirectoryMode, physical_test_root, write};

const TXT_IS_RUST: &str = "grund_config_version = 2\nproject_name = \"v\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\nexclude = [\"data\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n";

#[test]
fn a_target_read_as_no_value_source_still_fails_by_the_name_that_entered() {
    let root = physical_test_root("value_failed_status_no_format");
    write(&root.join("grund.toml"), TXT_IS_RUST);
    write(&root.join("data/x.txt"), "{\"CONST-a\": [\"a price\"]}\n");
    std::fs::create_dir_all(root.join("values")).expect("the home");
    let link = root.join("values/a.json");
    std::os::unix::fs::symlink("../data/x.txt", &link).expect("link the source");
    let config = load_config(&root).expect("the project loads");
    let (schema, frame) = (config.schema(), config.frame());
    assert!(frame.enters_as_value(schema, &link).is_some());
    assert_eq!(
        frame.reading(schema, &link).value_format(),
        None,
        "this run reads the target as no value source"
    );

    let _barred = DirectoryMode::set(&root.join("data"), 0o444);
    let denied = std::fs::metadata(&link)
        .is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied);
    if !denied {
        eprintln!("skipped: stating the file was not denied, as when running as root");
        return;
    }
    let (mut findings, mut errors) = (Catalog::default(), Vec::new());
    scan_value_sources(schema, frame, &mut findings, &mut errors);
    let expected: Vec<(PathBuf, String)> = vec![(
        link.clone(),
        "could not read value JSON source: permission denied".to_string(),
    )];
    assert_eq!(errors, expected);
}
