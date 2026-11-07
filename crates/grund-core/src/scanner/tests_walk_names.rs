//! A tree with no link pays nothing for a file's names (§FS-scan.identity.read-once,
//! §AR-scanner.1.8): the walk resolves as many paths whatever the tree's size, and asking how every
//! file is read resolves none.

use std::path::PathBuf;

use super::walk_scannable_files;
use crate::config::load_config;
use crate::sources::resolved_paths;
use crate::testing::{physical_test_root, write};

/// The paths this thread resolved over a walk and over reading each file the walk found, on a
/// linkless tree of `files` files.
fn resolved_over(case: &str, files: usize) -> (usize, usize, usize) {
    let root = physical_test_root(case);
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    for index in 0..files {
        write(&root.join(format!("docs/n{index}.md")), "Text.\n");
    }
    let config = load_config(&root).expect("the project loads");
    let (schema, frame) = (config.schema(), config.frame());
    let before = resolved_paths();
    let walked: Vec<PathBuf> = walk_scannable_files(schema, frame, None, false).expect("a walk");
    let after_walk = resolved_paths();
    for file in &walked {
        let _ = frame.reading(schema, file).markdown;
    }
    (
        walked.len(),
        after_walk - before,
        resolved_paths() - after_walk,
    )
}

#[test]
fn a_linkless_tree_resolves_no_more_for_more_files() {
    let (small_files, small_walk, small_reads) = resolved_over("names_count_small", 2);
    let (large_files, large_walk, large_reads) = resolved_over("names_count_large", 40);
    assert!(large_files >= 40 && large_files > small_files);
    assert_eq!(
        small_walk, large_walk,
        "the walk's resolutions follow the tree's size"
    );
    assert_eq!(
        (small_reads, large_reads),
        (0, 0),
        "the rule resolved a linkless file"
    );
}
