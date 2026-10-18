//! Logical copy paths shared by the runner and agreement (§FS-examples.5.6.2).
use std::path::PathBuf;

pub fn logical_copy(name: &str) -> PathBuf {
    PathBuf::from("target/e2e-work").join(name).join("repo")
}
