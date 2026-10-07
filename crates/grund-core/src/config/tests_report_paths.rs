//! Test module: which base a reported path is spelled from — the run's
//! `--path-base` over `[output] relative_paths`, the key alone, and the
//! `project` default (§FS-cli.3.4, §FS-config.3.6, §FS-config.3.6.1).

use std::path::PathBuf;

use super::report_paths::display_path;
use super::*;

/// A config rooted at `/ws` whose CLI base is `/ws/src/deep`, built inside
/// `base`'s scope the way a CLI run builds every config it loads.
fn config(relative_paths: bool, base: Option<PathBase>) -> Config {
    let mut config = with_report_path_base(base, || Config::default_for(PathBuf::from("/ws")));
    config.cli_base = PathBuf::from("/ws/src/deep");
    config.relative_paths = relative_paths;
    config
}

fn shown(config: &Config, path: &str) -> String {
    display_path(config, &PathBuf::from(path))
}

// §FS-config.3.6: with neither flag nor key, paths are spelled from the root.
#[test]
fn default_spells_paths_from_the_project_root() {
    let config = config(true, None);
    assert_eq!(shown(&config, "/ws/src/deep/lib.rs"), "src/deep/lib.rs");
    assert_eq!(shown(&config, "/ws/docs/FS-a.md"), "docs/FS-a.md");
}

// §FS-config.3.6.1: the key alone still moves the base to the CLI base,
// climbing with bounded `..` inside the loaded root.
#[test]
fn key_alone_spells_paths_from_the_cli_base() {
    let config = config(false, None);
    assert_eq!(shown(&config, "/ws/src/deep/lib.rs"), "lib.rs");
    assert_eq!(shown(&config, "/ws/docs/FS-a.md"), "../../docs/FS-a.md");
}

// §FS-cli.3.4: the flag outranks the key both ways.
#[test]
fn flag_overrides_the_key() {
    let invocation = config(true, Some(PathBase::Invocation));
    assert_eq!(shown(&invocation, "/ws/src/deep/lib.rs"), "lib.rs");
    assert_eq!(shown(&invocation, "/ws/docs/FS-a.md"), "../../docs/FS-a.md");

    let project = config(false, Some(PathBase::Project));
    assert_eq!(shown(&project, "/ws/src/deep/lib.rs"), "src/deep/lib.rs");
    assert_eq!(shown(&project, "/ws/docs/FS-a.md"), "docs/FS-a.md");
}

// §FS-cli.3.4: the scope ends with the call — a config built after it carries
// no override.
#[test]
fn the_override_ends_with_its_scope() {
    with_report_path_base(Some(PathBase::Invocation), || ());
    assert_eq!(Config::default_for(PathBuf::from("/ws")).path_base, None);
}

// §FS-cli.3.5: only the two documented values parse.
#[test]
fn only_project_and_invocation_parse() {
    assert_eq!(PathBase::parse("project"), Some(PathBase::Project));
    assert_eq!(PathBase::parse("invocation"), Some(PathBase::Invocation));
    assert_eq!(PathBase::parse("cwd"), None);
    assert_eq!(PathBase::parse(""), None);
}
