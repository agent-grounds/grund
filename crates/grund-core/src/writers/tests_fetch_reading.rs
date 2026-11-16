//! A source declaration discovered by the one pre-integration scan refuses the fetch before
//! the integration runs, even when its name ends in `.md` (§FS-fetch.5.2).

use super::fetch_snapshot;
use crate::testing::{test_root, write};
use std::os::unix::fs::PermissionsExt;

#[test]
fn a_source_reading_declares_before_the_integration_runs() {
    let root = test_root("fetch_source_reading_before_spawn");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"python\"]\n\
         [schema.sources.extensions]\nmd = \"python\"\n[schema.kinds.FS]\n\
         folders = [\"fs\"]\nindex = false\nfetch = \"./fetcher\"\n",
    );
    let original = "\"\"\"FS-a: Alpha\n\nLocal body.\n\"\"\"\nx = 1\n";
    write(&root.join("fs/a.md"), original);
    write(
        &root.join("fetcher"),
        "#!/bin/sh\ntouch spawned\nprintf '# FS-a: Alpha\\n'\n",
    );
    std::fs::set_permissions(root.join("fetcher"), std::fs::Permissions::from_mode(0o755))
        .expect("executable integration");
    let error = fetch_snapshot("FS-a", &root).expect_err("the source file already declares it");
    assert!(
        error.message.contains("fs/a.md already declares it"),
        "{error}"
    );
    assert!(!root.join("spawned").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("fs/a.md")).unwrap(),
        original
    );
}
