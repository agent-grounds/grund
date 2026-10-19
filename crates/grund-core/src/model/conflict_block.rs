//! The conflict blocks the file pass records (§AR-scanner.2.9). What a block is,
//! where it is looked for and whom it names is §FS-declarations.body.conflict;
//! this is the one record `check`'s finding and `fmt`'s refusal both read.

use std::path::PathBuf;

use super::records::Id;

/// One conflict block found where §FS-declarations.body.conflict.where looks: its
/// first and last lines (§FS-declarations.body.conflict.block), equal for a lone
/// closer, and the declarations it names, each once, in the order they open
/// (§FS-declarations.body.conflict.owners). Empty owners is a block in a declaring
/// Markdown file that overlaps no body.
#[derive(Clone, Debug)]
pub(crate) struct ConflictBlock {
    pub(crate) file: PathBuf,
    pub(crate) first: usize,
    pub(crate) last: usize,
    pub(crate) owners: Vec<Id>,
}
