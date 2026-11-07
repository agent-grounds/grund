//! The roots a run reads and the homes it does not, as one question a path can ask
//! (§FS-check.1.3, §FS-config.3.4.7.5). It sits in `config` because a file's names are judged
//! against it in the walk (§AR-scanner.1.8), and a report is narrowed by it in the
//! checker (§FS-check.1.3.6.1), so neither of those owns it (§AR-config.1.6).

use std::path::{Path, PathBuf};

use super::scope_roots::{root_scope_roots, unwalked_home_roots};
use super::{Nesting, Schema};
use crate::model::{scanned_decl_relative_path, scanned_path_key};
use crate::sources::physical_path_key;

/// A set of roots, less the homes a run without `--full` never reads.
pub(crate) struct WalkBounds {
    roots: Vec<PathBuf>,
    /// The `scan = false` homes under those roots (§FS-config.3.4.7.5): listed by
    /// the config, and read by this run only because `--full` widened the walk.
    unwalked: Vec<PathBuf>,
    /// v2 only: the schema whose ownership rule settles a path under an unwalked
    /// place, and the config root's two spellings to ask it relative to.
    owners: Option<PlaceOwners>,
}

/// [`Schema::owner_walked`] kept with the scope (§FS-config-v2.schema.places.boundaries.6).
struct PlaceOwners {
    schema: Schema,
    configured_root: PathBuf,
    physical_root: PathBuf,
}

impl WalkBounds {
    pub(crate) fn new(
        schema: &Schema,
        root: &Path,
        roots: Vec<PathBuf>,
        unwalked: Vec<PathBuf>,
    ) -> Self {
        let owners =
            (schema.nesting == Nesting::DeepestPlaceWins && !unwalked.is_empty()).then(|| {
                PlaceOwners {
                    schema: schema.clone(),
                    configured_root: scanned_path_key(root),
                    physical_root: physical_path_key(root),
                }
            });
        Self {
            roots,
            unwalked,
            owners,
        }
    }

    /// The names a plain run walks, as written (§AR-scanner.1.8). It holds no canonical form,
    /// which would count `real/a.md` as reached where only `include = ["link"]` reaches that file
    /// (§FS-check.1.3.2).
    pub(crate) fn plain(schema: &Schema, root: &Path) -> Self {
        let roots = root_scope_roots(schema, root, false);
        Self::new(schema, root, roots, unwalked_home_roots(schema, root))
    }

    pub(crate) fn contains(&self, path: &Path) -> bool {
        // §FS-config.3.4.7.5: a file in a home the config lists without walking is
        // outside the configured scope even when a root above it is inside — the
        // scope is a set of roots, less the homes a run without `--full` never reads.

        // In v2 the place that owns the file decides, so a scanned place under an
        // unscanned one stays in scope (§FS-config-v2.schema.places.boundaries.6).
        if self.unwalked.iter().any(|home| path.starts_with(home)) {
            let walked = self.owners.as_ref().is_some_and(|owners| {
                let roots = (&owners.configured_root, &owners.physical_root);
                scanned_decl_relative_path(path, roots.0, roots.1)
                    .is_some_and(|relative| owners.schema.owner_walked(&relative))
            });
            if !walked {
                return false;
            }
        }
        self.roots
            .iter()
            .any(|root| path == root || path.starts_with(root))
    }
}
