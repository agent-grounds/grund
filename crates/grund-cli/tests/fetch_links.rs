//! What one case directory cannot hold about a link at a fetch target (§FS-fetch.links) and the
//! paths a refusal names (§FS-fetch.7): a link target outside the copied tree, which a case cannot
//! make; a dangling link under `md = "json"`, where §FS-fetch.4.2.1's reason for a data mapping is
//! not settled, so only its shape is pinned; and a mode bit git does not carry.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use support::*;

const OLD: &str = "# T-1: Old\n\nThe old body.\n";
const OUT: &str = "# T-1: Fetched\n\nThe fetched body.\n";
const FETCHER: &str = "#!/bin/sh\nprintf '# %s: Fetched\\n\\nThe fetched body.\\n' \"$1\"\n";
const FILE_FETCHER: &str = "#!/bin/sh\nprintf '## %s: Fetched\\n\\nThe fetched body.\\n' \"$1\"\n";

const FOLDER: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                      [schema.kinds.T]\nfolders = [\"docs/t\"]\nid_format = \"{kind}-{number}\"\n\
                      index = false\nfetch = \"scripts/fetch\"\n";
const FOLDER_MD_JSON: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                              [schema.sources]\nlanguages = [\"markdown\", \"json\"]\n\n\
                              [schema.sources.extensions]\nmd = \"json\"\n\n\
                              [schema.kinds.T]\nfolders = [\"docs/t\"]\n\
                              id_format = \"{kind}-{number}\"\nindex = false\n\
                              fetch = \"scripts/fetch\"\n";
const FILE: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                    [schema.kinds.T]\nfiles = [\"docs/t.md\"]\nid_format = \"{kind}-{number}\"\n\
                    fetch = \"scripts/fetch\"\n";

fn outward(path: &str) -> String {
    format!(
        "error: cannot fetch T-1 into {path}: the symlink target is outside the config root; \
         replace the link with the file it points to, and fetch again\n"
    )
}

/// A project at `<base>/proj` beside `<base>/outside`, which a link can leave the root for.
fn project(test: &str, config: &str, fetcher: &str) -> (PathBuf, PathBuf) {
    let base = root(&format!("links-{test}"));
    let proj = base.join("proj");
    write(&proj, "grund.toml", config);
    executable(&proj, "scripts/fetch", fetcher);
    fs::create_dir_all(proj.join("docs/t")).expect("create home");
    fs::create_dir_all(base.join("outside")).expect("create outside");
    (base, proj)
}

fn link(proj: &Path, at: &str, target: &str) {
    symlink(target, proj.join(at)).expect("create link");
}

/// Every entry under `base` without following a link: a link reads as its target's name, so a
/// cut link or a write through it both show.
fn snapshot(base: &Path) -> BTreeMap<String, String> {
    fn collect(base: &Path, at: &Path, out: &mut BTreeMap<String, String>) {
        let mut entries = fs::read_dir(at)
            .expect("read fixture tree")
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let key = path
                .strip_prefix(base)
                .expect("fixture-relative path")
                .to_string_lossy()
                .into_owned();
            let meta = fs::symlink_metadata(&path).expect("entry metadata");
            if meta.file_type().is_symlink() {
                let target = fs::read_link(&path).expect("read link");
                out.insert(key, format!("link -> {}", target.display()));
            } else if meta.is_dir() {
                out.insert(format!("{key}/"), String::new());
                collect(base, &path, out);
            } else {
                let bytes = fs::read(&path).expect("read fixture file");
                out.insert(key, String::from_utf8_lossy(&bytes).into_owned());
            }
        }
    }
    let mut out = BTreeMap::new();
    collect(base, base, &mut out);
    out
}

/// Fetch `T-1` in `proj`, and hold the refusal to exit 2, empty stdout and `base` unchanged.
fn refused(base: &Path, proj: &Path, context: &str) -> String {
    let before = snapshot(base);
    let fetch = run(proj, &["fetch", "T-1"]);
    assert_code(&fetch, 2, context);
    assert_eq!(stdout(&fetch), "", "{context}: stdout");
    assert_eq!(snapshot(base), before, "{context}: tree");
    stderr(&fetch)
}

#[test]
fn links_an_outward_link_in_a_folder_home_is_refused() {
    let (base, proj) = project("outward-folder", FOLDER, FETCHER);
    fs::write(base.join("outside/T-1.md"), OLD).expect("write target");
    link(&proj, "docs/t/T-1.md", "../../../outside/T-1.md");
    let listed = run(&proj, &["list", "--kind", "T"]);
    assert!(stdout(&listed).contains("T-1"), "the walk reads the link");
    let err = refused(&base, &proj, "outward link");
    assert_eq!(err, outward("docs/t/T-1.md"));
}

#[test]
fn links_an_outward_link_already_holding_the_output_is_refused() {
    let (base, proj) = project("outward-same", FOLDER, FETCHER);
    fs::write(base.join("outside/T-1.md"), OUT).expect("write target");
    link(&proj, "docs/t/T-1.md", "../../../outside/T-1.md");
    let err = refused(&base, &proj, "outward link, same bytes");
    assert_eq!(err, outward("docs/t/T-1.md"));
}

#[test]
fn links_a_file_home_that_is_an_outward_link_is_refused() {
    let (base, proj) = project("outward-file", FILE, FILE_FETCHER);
    let home = "# Tickets\n\n## T-1: Old\n\nThe old body.\n\n## T-2: Other\n\nOther body.\n";
    fs::write(base.join("outside/t.md"), home).expect("write target");
    link(&proj, "docs/t.md", "../../outside/t.md");
    let err = refused(&base, &proj, "outward file home");
    assert_eq!(err, outward("docs/t.md"));
}

#[test]
fn links_a_dangling_link_under_md_json_is_refused_as_unread() {
    let (base, proj) = project("dangling-md-json", FOLDER_MD_JSON, FETCHER);
    link(&proj, "docs/t/T-1.md", "../../gone.md");
    let err = refused(&base, &proj, "dangling link under md = json");
    // §FS-fetch.4.2.1 speaks first; its <reason> for a data mapping is not settled, so no more.
    assert!(
        err.starts_with("error: kind `T` cannot fetch into "),
        "unread refusal expected, got {err:?}"
    );
}

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set fixture mode");
}

/// A superuser reads a mode-000 path, so such a fixture proves nothing there.
fn mode_bars(path: &Path) -> bool {
    fs::read(path).is_err() && fs::read_dir(path).is_err()
}

/// Refuse with `path` at `mode`, restore the mode, and return stderr, or `None` where the mode
/// does not bar this user.
fn refused_at_mode(base: &Path, proj: &Path, path: &Path, context: &str) -> Option<String> {
    let before = snapshot(base);
    set_mode(path, 0o000);
    if !mode_bars(path) {
        set_mode(path, 0o755);
        return None;
    }
    let fetch = run(proj, &["fetch", "T-1"]);
    set_mode(path, 0o755);
    assert_code(&fetch, 2, context);
    assert_eq!(stdout(&fetch), "", "{context}: stdout");
    assert_eq!(snapshot(base), before, "{context}: tree");
    Some(stderr(&fetch))
}

#[test]
fn links_cannot_read_snapshot_folder_names_paths_from_the_report_base() {
    let (base, proj) = project("cannot-read-home", FOLDER, FETCHER);
    let home = proj.join("docs/t");
    if let Some(err) = refused_at_mode(&base, &proj, &home, "unreadable home") {
        assert_eq!(
            err,
            "error: cannot read snapshot folder docs/t at docs/t: permission denied\n"
        );
    }
    let (base, proj) = project("cannot-read-subfolder", FOLDER, FETCHER);
    let locked = proj.join("docs/t/locked");
    fs::create_dir(&locked).expect("create subfolder");
    if let Some(err) = refused_at_mode(&base, &proj, &locked, "unreadable subfolder") {
        assert_eq!(
            err,
            "error: cannot read snapshot folder docs/t at docs/t/locked: permission denied\n"
        );
    }
}

#[test]
fn links_cannot_read_a_home_file_names_it_from_the_report_base() {
    let (base, proj) = project("cannot-read-file", FOLDER, FETCHER);
    write(&proj, "docs/t/other.md", "# T-8: Another ticket\n");
    let other = proj.join("docs/t/other.md");
    if let Some(err) = refused_at_mode(&base, &proj, &other, "unreadable file") {
        assert_eq!(
            err,
            "error: cannot read docs/t/other.md: Permission denied (os error 13)\n"
        );
    }
}
