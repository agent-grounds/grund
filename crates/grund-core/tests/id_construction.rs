//! Since the scheduled construction change an embedder reads the public identity
//! record from what an entry point returns and ends a pattern on it in `..`
//! (§FS-distribution.3.1.2), while presentation lives outside identity
//! (§FS-config.3.2.8).

#[path = "support/numeric_spelling.rs"]
mod support;

use grund_core::{Id, scan};
use support::Repo;

#[test]
fn public_id_is_read_from_a_scan_with_its_three_fields() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            let catalog = scan(&repo.0).unwrap();
            let ids: Vec<&Id> = catalog.declarations.keys().collect();
            assert_eq!(ids.len(), 1, "v{version}, per kind {per_kind}");
            // `ADR-0001` is written with its zeros; identity keeps the number alone.
            let Id {
                kind, num, slug, ..
            } = ids[0];
            assert_eq!(kind, "ADR");
            assert_eq!(*num, Some(1));
            assert_eq!(*slug, None);
        }
    }
}
