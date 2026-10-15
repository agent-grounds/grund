//! Stable ordering of the walk's path lists (§AR-scanner.1.12), using the same
//! formatted key as every report (§FS-errors.4.1).

use std::path::PathBuf;

use crate::model::sort_path_key;

/// Sort one walk list while preserving the input order of equal formatted keys
/// (§AR-scanner.1.12), constructing each key once for this sort alone.
/// Distinct spellings with the same lossy key keep their input order.
pub(super) fn sort_paths(paths: &mut [PathBuf]) {
    paths.sort_by_cached_key(|path| {
        let key = sort_path_key(path);
        #[cfg(test)]
        tests::key_created();
        key
    });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::PathBuf;

    use super::sort_paths;

    thread_local! {
        static KEYS_CREATED: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn key_created() {
        KEYS_CREATED.with(|count| count.set(count.get() + 1));
    }

    /// Observe actual formatted-key construction, not a sorter policy flag
    /// (§AR-scanner.1.12). The permutation keeps the input nontrivial.
    #[test]
    fn constructs_one_formatted_key_per_element() {
        let mut paths: Vec<_> = (0..64)
            .map(|index| PathBuf::from(format!("docs/file-{:02}.md", (index * 37) % 64)))
            .collect();
        let expected: Vec<_> = (0..64)
            .map(|index| PathBuf::from(format!("docs/file-{index:02}.md")))
            .collect();

        KEYS_CREATED.with(|count| count.set(0));
        sort_paths(&mut paths);
        let created = KEYS_CREATED.with(Cell::get);

        assert_eq!(paths, expected);
        assert_eq!(
            created,
            paths.len(),
            "the walk reconstructed formatted keys while comparing unchanged paths"
        );
    }

    /// Normalizing a backslash and a slash can give distinct paths equal keys;
    /// the root's first-seen spelling must still win (§AR-scanner.1.12).
    #[test]
    fn keeps_input_order_for_equal_slash_normalized_keys() {
        let mut paths = vec![
            PathBuf::from("z.md"),
            PathBuf::from(r"docs\a.md"),
            PathBuf::from("docs/a.md"),
            PathBuf::from("A.md"),
        ];

        sort_paths(&mut paths);

        assert_eq!(
            paths,
            [
                PathBuf::from("A.md"),
                PathBuf::from(r"docs\a.md"),
                PathBuf::from("docs/a.md"),
                PathBuf::from("z.md"),
            ]
        );
    }

    /// Lossy display can collapse two Unix filenames to the same string; keys
    /// must not replace the paths or change their stable order (§AR-scanner.1.12).
    #[cfg(unix)]
    #[test]
    fn keeps_distinct_non_utf8_paths_in_input_order() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(b"docs/\xff.md".to_vec()));
        let second = PathBuf::from(OsString::from_vec(b"docs/\xfe.md".to_vec()));
        let mut paths = vec![PathBuf::from("z.md"), first.clone(), second.clone()];

        sort_paths(&mut paths);

        assert_eq!(paths, [first, second, PathBuf::from("z.md")]);
    }

    #[test]
    fn empty_and_singleton_lists_keep_their_contents() {
        let mut empty = Vec::new();
        sort_paths(&mut empty);
        assert!(empty.is_empty());

        let mut singleton = vec![PathBuf::from("one.md")];
        sort_paths(&mut singleton);
        assert_eq!(singleton, [PathBuf::from("one.md")]);
    }
}
