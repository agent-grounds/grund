//! The canonicalizing boundary must retain ownership too (§FS-repository-maintenance.4.1).

use super::physical_test_root as test_root;

#[path = "fixture_root_contract.rs"]
mod fixture_root_contract;

#[test]
fn fixture_root_contract_path_consuming_constructor_keeps_owner() {
    let path;
    {
        let root = test_root("fixture_root_contract_path_consuming_constructor_keeps_owner");
        path = root.to_path_buf();
        let config = crate::config::Config::default_for(root.to_path_buf());
        assert_eq!(config.root, std::fs::canonicalize(&path).unwrap());
        std::fs::write(config.root.join("sentinel"), "consumer uses canonical root").unwrap();
        drop(config);
        assert_eq!(
            std::fs::read_to_string(root.join("sentinel")).unwrap(),
            "consumer uses canonical root"
        );
    }
    assert!(
        !path.exists(),
        "path consumer left owned root: {}",
        path.display()
    );
}
