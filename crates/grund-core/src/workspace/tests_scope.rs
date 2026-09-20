//! Test module: workspace root-scope classification (§AR-core-module-layout.1.5)

use std::path::Path;

use super::scope_is_config_root;
use crate::config::Config;
use crate::testing::{canonical_test_path, test_root};

#[test]
fn workspace_root_scope_requires_canonical_root_for_explicit_path() {
    let root = canonical_test_path(&test_root(
        "workspace_root_scope_requires_canonical_root_for_explicit_path",
    ));
    let subdir = root.join("apps/api");
    std::fs::create_dir_all(&subdir).expect("create subdir");
    let config = Config::default_for(root.clone());

    assert!(scope_is_config_root(&config, Path::new("."), false));
    assert!(scope_is_config_root(&config, &root, true));
    assert!(
        !scope_is_config_root(&config, &subdir, true),
        "an explicit subdirectory scope must not be promoted to workspace root"
    );
}
