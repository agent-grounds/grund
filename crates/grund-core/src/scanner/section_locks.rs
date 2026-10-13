//! The section lock files a run judges (§FS-section-lock.file.shard): read once
//! beside the walk, which never enters `.grund/`, so a lock is never a source and
//! `check` reads no file after the scan to judge one (§AR-checker.placement).

use std::path::{Path, PathBuf};

use crate::config::Frame;
use crate::grammar::{STUB_LINK_HEADING, may_hold_relative_heading, mixed_heading_path};
use crate::model::{Catalog, SECTION_LOCK_DIR, SectionLockFile};

/// Whether the config root holds anything under `.grund/sections/`, read where
/// `read_section_locks` reads: what makes a run that has not scanned ask for a
/// scan of the lock pass (§FS-section-lock.write).
pub(crate) fn section_locks_present(frame: Frame<'_>) -> bool {
    frame
        .sources()
        .read_dir(frame.root().join(SECTION_LOCK_DIR))
        .is_ok_and(|mut entries| entries.next().is_some())
}

/// Whether the walked Markdown file `path`, holding `text`, may give the scan a
/// heading whose address is locked once cited (§FS-section-lock.holds.entry): a
/// relative heading, or with `named_sections` a heading whose path is mixed, of its
/// own or in the Markdown file a stub-shaped heading in it links to, which the
/// scan reads on the stub's behalf wherever it lies (§FS-declarations.stubs.target.5).
/// With `section_locks_present`, what makes a run that has not scanned ask for a
/// scan of the lock pass (§FS-section-lock.write): it may answer yes where the
/// scan finds no locked address, and never no where it finds one, because the
/// judgement it stands in for is made on the scan alone.
pub(crate) fn may_give_locked_heading(frame: Frame<'_>, path: &Path, text: &str) -> bool {
    let sources = frame.sources();
    let named = frame.grammar().named_sections;
    let may_hold = |text: &str| {
        may_hold_relative_heading(text) || named && text.lines().any(is_mixed_heading_line)
    };
    // The scan reads a stub's link off the text after its ID, which ends at a
    // colon, so every colon of a heading line is tried.
    let linked = |line: &str| {
        if !line.trim_start().starts_with('#') {
            return None;
        }
        line.match_indices(':')
            .find_map(|(at, _)| Some(STUB_LINK_HEADING.captures(&line[at..])?["path"].to_owned()))
    };
    may_hold(text)
        || text.lines().filter_map(linked).any(|target| {
            let target = Path::new(&target);
            target.extension().and_then(|ext| ext.to_str()) == Some("md")
                && sources
                    .text(&sources.stub_target(frame.root(), path, target))
                    .is_ok_and(|text| may_hold(&text))
        })
}

/// Whether `line` is an ATX heading whose text reads as a complete-path heading at
/// a mixed path (§FS-config.3.3.1), wherever it sits: a superset of the headings
/// the scan records at one.
fn is_mixed_heading_line(line: &str) -> bool {
    let line = line.trim_start_matches(' ');
    let after = line.trim_start_matches('#');
    after.len() < line.len()
        && after.starts_with([' ', '\t'])
        && mixed_heading_path(after.trim_start_matches([' ', '\t'])).is_some()
}

/// Every `<ID>.lock` directly under the config root's `.grund/sections/`, and
/// every one under a directory there named by an alias, for the citations into
/// that project (§FS-section-lock.file.members), sorted by path. A tree with no
/// such directory costs one failed listing. A file that cannot be read joins the
/// run's unreadable paths (§FS-check.2.4).
pub(super) fn read_section_locks(
    frame: Frame<'_>,
    findings: &mut Catalog,
    errors: &mut Vec<(PathBuf, String)>,
) {
    let dir = frame.root().join(SECTION_LOCK_DIR);
    let mut paths = Vec::new();
    for path in listing(frame, &dir) {
        if path.extension().and_then(|ext| ext.to_str()) == Some("lock") {
            paths.push((None, path));
        } else if let Some(alias) = path.file_name().and_then(|name| name.to_str()) {
            let alias = alias.to_string();
            paths.extend(
                listing(frame, &path)
                    .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lock"))
                    .map(|path| (Some(alias.clone()), path)),
            );
        }
    }
    paths.sort_by(|(_, a), (_, b)| a.cmp(b));
    for (alias, path) in paths {
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        match frame.sources().text(&path) {
            Ok(text) => findings.section_locks.push(SectionLockFile {
                alias,
                id: id.to_string(),
                text: text.into_owned(),
            }),
            Err(error) => errors.push((path, format!("could not read section lock: {error}"))),
        }
    }
}

/// The paths directly under `dir`, none where it cannot be listed.
fn listing(frame: Frame<'_>, dir: &Path) -> impl Iterator<Item = PathBuf> {
    frame
        .sources()
        .read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
}
