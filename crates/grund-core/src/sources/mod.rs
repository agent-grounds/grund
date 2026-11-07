//! The sources component (§AR-system.2.13): the one part of the engine that
//! reads the disk. For a path it answers the text a save would write, whether
//! the file exists, whether it is a file, its listing and its physical
//! identity, with the run's buffers in front of the disk
//! (§FS-scan.text.buffer, §FS-scan.text.buffer-only, §FS-scan.identity.buffers).
//! It knows no grammar, no configuration and no rule; what a file's text means
//! is the scanner's.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). `observe.rs` is the run's input observer, which
//! came down out of `model/check_inputs.rs`; `identity.rs` the physical path
//! keys, out of `model/paths.rs`; `disk.rs` the reads, out of `config/inputs.rs`;
//! and `buffers.rs` the buffers and the view every reader asks. `native_watch.rs`
//! is the disk's change notification: the debounce its events are coalesced by
//! (§FS-check.6.1.2), and behind the `native-watch` feature its backend, which only
//! the frontends that wait on the disk turn on (§AR-lsp.2.1).

mod buffers;
mod digest;
mod disk;
mod identity;
pub mod native_watch;
mod observe;

pub use identity::canonical_snapshot_path;
pub use observe::{CheckInput, CheckInputObserver, with_check_input_observer};

// What the other components read, each by this module's path (§AR-system.4).
pub(crate) use buffers::{Buffers, Sources};
pub(crate) use digest::sha256_hex;
pub(crate) use disk::{
    canonical_path, input_read_to_string, is_directory, is_regular_file, link_metadata,
    path_exists, path_is_dir, path_is_entry, path_is_file, path_is_symlink, path_metadata,
    read_bytes, read_git_metadata, read_link,
};
pub(crate) use identity::{canonicalize_existing_prefix, paths_same_location, physical_path_key};
pub(crate) use observe::{check_input_observer, observe_input, record_test_workspace_load};
