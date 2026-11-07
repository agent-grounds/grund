//! A tree with no link pays nothing for a file's names (§FS-scan.identity.read-once,
//! §AR-scanner.1.8): the walk resolves as many paths whatever the tree's size, and asking how every
//! file is read resolves none.

use std::path::PathBuf;

use super::walk_scannable_files;
use crate::config::load_config;
use crate::sources::realpaths_during;
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
    let (walked, walk): (Vec<PathBuf>, usize) =
        realpaths_during(|| walk_scannable_files(schema, frame, None, false).expect("a walk"));
    let ((), reads) = realpaths_during(|| {
        for file in &walked {
            let _ = frame.reading(schema, file).markdown;
        }
    });
    (walked.len(), walk, reads)
}

/// The count is of `physical_path_key` and `canonical_path` on this thread, and nothing wider: a
/// scan's rayon workers go past it, so this pins the walk and the rule, not a run's every realpath.
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

/// The route `check <value source>` takes on a linkless tree: the value scan asks where the source
/// is, a whole walk then publishes, and the scan asks again. A run resolves a spelling once, so the
/// second ask costs nothing. This thread's realpath count is all that sees it; the
/// route's whole `readlink` count was measured under `strace` and is not something a test can pin.
#[test]
fn a_whole_walk_keeps_where_a_spelling_was_resolved() {
    let root = physical_test_root("names_count_value_route");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    let source = root.join("values/v1.json");
    write(&source, "{}\n");
    let config = load_config(&root).expect("the project loads");
    let (schema, frame) = (config.schema(), config.frame());
    let spellings = &frame.run.spellings;

    let first = spellings.resolve(&source);
    walk_scannable_files(schema, frame, None, false).expect("a walk");
    let (again, spent) = realpaths_during(|| spellings.resolve(&source));
    assert_eq!(again, first);
    assert_eq!(spent, 0, "the walk forgot what was resolved");
}
