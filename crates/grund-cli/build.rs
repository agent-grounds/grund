//! Records the version `grund --version` reports as `GRUND_BUILD_VERSION`,
//! derived once in `build_identity.rs` (§FS-distribution.4.1.1).

#[path = "build_identity.rs"]
mod build_identity;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let version = env::var("CARGO_PKG_VERSION").expect("Cargo sets CARGO_PKG_VERSION");
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets it"));
    // Releases never consult the variable; development builds watch it even
    // where git wins, which still refuses a malformed one (§FS-distribution.4.1.1).
    let commit = if version.ends_with("-dev") {
        println!("cargo:rerun-if-env-changed=GRUND_BUILD_COMMIT");
        env::var_os("GRUND_BUILD_COMMIT")
    } else {
        None
    };
    match build_identity::build_identity(&version, &crate_dir, commit.as_deref(), OsStr::new("git"))
    {
        Ok(identity) => {
            for path in &identity.watched {
                println!("cargo:rerun-if-changed={}", path.display());
            }
            println!("cargo:rustc-env=GRUND_BUILD_VERSION={}", identity.version);
            ExitCode::SUCCESS
        }
        Err(refused) => {
            eprintln!("error: {refused}");
            ExitCode::FAILURE
        }
    }
}
