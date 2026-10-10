//! From read `[[kinds]]` entries to the schema's rows (§AR-config.3.1): first
//! the refusals of an entry no row of §AR-config.1.3 can hold, then one row
//! each, in file order, and the title, grounding pair and `resolve` each entry
//! wrote into the concern that holds it. The two per-name defaults are applied
//! here, in v1 and nowhere else (§AR-config.3.2): a declared folder kind picks
//! up the `index` its name has built in, and a fetched kind resolves at `must`
//! unless it says otherwise (§FS-config.3.4.10). `kind_table.rs` reads the
//! entries' keys.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::path::Path;

use super::kind_defaults::default_kind_index;
use super::kind_table::ParsedKind;
use super::kind_values::validate_kind_value_keys;
use crate::config::kind::{KindConfig, KindIndex, KindResolution};
use crate::config::project::{KindGrounding, Project};
use crate::config::record::{CODE_SOURCE_KIND, ConfigLocation};
use crate::config::rows::{Extent, Form, Kind, Origin, Place, Row};
use crate::model::format_path;

/// Every rule a `[[kinds]]` entry has to satisfy before it can be a row
/// (§FS-config.3.4): each entry names a kind, `folder` and `file` are
/// exclusive, `index` needs a folder and a citable kind, a rule or value kind
/// has the home it needs, `code` is reserved, and an unwalked place declares
/// nothing. Each closes a combination the row of §AR-config.1.3 cannot hold, so
/// the v1 reader refuses it while it still has the spelling to name. Applied
/// only when the file declared the block at all — `[[kinds]]` replaces the
/// built-in rows entirely rather than merging into them.
///
/// The entries then lower into `project`, in `lower_rows` below
/// (§AR-config.3.1).
pub(super) fn lower_parsed_kinds(
    path: &Path,
    root: &Path,
    parsed: Vec<ParsedKind>,
    project: &mut Project,
) -> Result<()> {
    if let Some(nameless) = parsed.iter().find(|entry| entry.config.kind.is_empty()) {
        return Err(anyhow!(
            "{}:{}: every [[kinds]] entry must declare a `kind`",
            format_path(path),
            nameless.header_line
        ));
    }
    if parsed.is_empty() {
        return Err(anyhow!(
            "{}: at least one [[kinds]] entry must declare a `kind`",
            format_path(path)
        ));
    }
    for entry in &parsed {
        let k = &entry.config;
        let rule_file_is_markdown = k.file.as_deref().is_none_or(|file| {
            Path::new(file).extension().and_then(|ext| ext.to_str()) == Some("md")
        });
        if k.rules
            && (!k.citable
                || !k.scan
                || (k.folder.is_none() && k.file.is_none())
                || !rule_file_is_markdown
                || k.values
                || k.fetch.is_some())
        {
            return Err(anyhow!(
                "{}: kind `{}` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`",
                format_path(path),
                k.kind
            ));
        }
        // §FS-config.3.4.10: an override belongs only to an ID namespace; an
        // explicit obligation needs the fetch remedy, and fetching needs one
        // unambiguous snapshot home.
        if k.format.is_some() && !k.citable {
            return Err(anyhow!(
                "{}: kind `{}` sets `format` with `citable = false`",
                format_path(path),
                k.kind
            ));
        }
        if k.resolve.is_some() && k.fetch.is_none() {
            return Err(anyhow!(
                "{}: kind `{}` sets `resolve` but requires `fetch`",
                format_path(path),
                k.kind
            ));
        }
        if k.fetch.is_some() {
            if !k.citable {
                return Err(anyhow!(
                    "{}: kind `{}` sets `fetch` with `citable = false`",
                    format_path(path),
                    k.kind
                ));
            }
            if usize::from(k.file.is_some()) + usize::from(k.folder.is_some()) != 1 {
                return Err(anyhow!(
                    "{}: kind `{}` sets `fetch` without exactly one `file` or `folder` home",
                    format_path(path),
                    k.kind
                ));
            }
        }
        // Reject kinds that set both `folder` and `file` — they're mutually exclusive
        // (§FS-config.3.4): a kind is either multi-file (folder) or single-file
        // (file), and "can always be broken up" swaps one key for the other.
        if k.folder.is_some() && k.file.is_some() {
            return Err(anyhow!(
                "{}: kind `{}` sets both `folder` and `file` (use one)",
                format_path(path),
                k.kind
            ));
        }
        // §FS-config.3.4: `index` names a file inside `folder`, so a kind
        // with no folder — a single-file `file` kind, or one with no home at
        // all — has nothing to index and the key is a config error.
        if k.index != KindIndex::Default && k.folder.is_none() {
            return Err(anyhow!(
                "{}: kind `{}` sets `index` without `folder` (only a folder kind has an index)",
                format_path(path),
                k.kind
            ));
        }
        // §FS-config.3.4: an index lists the folder's declarations
        // (§FS-check.3.18.1), and a non-citable kind has none — so the key is not a
        // no-op here, it is a statement about a set that can never be non-empty.
        if k.index != KindIndex::Default && !k.citable {
            return Err(anyhow!(
                "{}: kind `{}` sets `index` and `citable = false` (a non-citable kind declares nothing to index)",
                format_path(path),
                k.kind
            ));
        }
        // §FS-config.3.9.2.2: `code` is the *default* name of the homeless kind, and a
        // name a project may take only by declaring that kind — the complement of
        // every home. Any other row wearing it would collide with that fallback.
        if k.kind == CODE_SOURCE_KIND && !(!k.citable && k.folder.is_none() && k.file.is_none()) {
            return Err(anyhow!(
                "{}: `{CODE_SOURCE_KIND}` names the homeless kind — a [[kinds]] entry may take it only with `citable = false` and no `folder` or `file`",
                format_path(path)
            ));
        }
        validate_kind_value_keys(path, entry, root, project.schema.ids.named_sections)?;
    }
    // §FS-config.3.4.7.6: an unwalked kind is a place and nothing more. A citable one
    // would have declarations nobody reads — the trap §FS-config.3.5.10 closes — and the
    // homeless kind has no home to leave unwalked (`[scan] include` says what is read).
    for entry in &parsed {
        let k = &entry.config;
        if k.scan {
            continue;
        }
        if k.citable {
            return Err(anyhow!(
                "{}: kind `{}` sets `scan = false` and declares IDs (an unwalked kind's declarations would be invisible — set `citable = false`, or drop the key)",
                format_path(path),
                k.kind
            ));
        }
        if k.folder.is_none() && k.file.is_none() {
            return Err(anyhow!(
                "{}: kind `{}` sets `scan = false` without a home (what of the homeless kind is walked is `[scan] include`'s to say)",
                format_path(path),
                k.kind
            ));
        }
    }
    lower_rows(path, &parsed, project);
    Ok(())
}

/// Lower `parsed` into `project`, replacing the built-in rows and every
/// per-kind fact they carried: a declared `[[kinds]]` table replaces them all
/// (§FS-config.3.4).
fn lower_rows(path: &Path, parsed: &[ParsedKind], project: &mut Project) {
    let mut titles = BTreeMap::new();
    let mut grounding = BTreeMap::new();
    let mut resolution = BTreeMap::new();
    let rows = parsed
        .iter()
        .map(|entry| {
            let k = &entry.config;
            // A name another row already wrote keeps the first row's facts; the
            // second row is refused as a duplicate once the table is lowered.
            if let Some(title) = &k.title {
                titles
                    .entry(k.kind.clone())
                    .or_insert_with(|| title.clone());
            }
            if k.require_grounding.is_some() || k.grounding_level.is_some() {
                let at = |line: Option<usize>| {
                    line.map(|line| ConfigLocation {
                        path: path.to_path_buf(),
                        line,
                    })
                };
                grounding
                    .entry(k.kind.clone())
                    .or_insert_with(|| KindGrounding {
                        require: k.require_grounding,
                        require_source: at(entry.grounding.require_line),
                        level: k.grounding_level,
                        level_source: at(entry.grounding.level_line),
                        ladder: None,
                    });
            }
            let resolve = k
                .resolve
                .or(k.fetch.is_some().then_some(KindResolution::Must));
            if let Some(resolve) = resolve {
                resolution.entry(k.kind.clone()).or_insert(resolve);
            }
            lower_row(k)
        })
        .collect();
    project.schema.rows = rows;
    project.presentation.kinds = titles;
    project.rules.grounding.kinds = grounding;
    project.rules.resolution = resolution;
}

/// One validated entry as a row of §AR-config.1.3: its one place, and its kind
/// when it declares IDs. The complement is the non-citable entry with no home
/// (§FS-config.3.9.2.1); a citable entry with no home is a kind with no place.
fn lower_row(k: &KindConfig) -> Row {
    let extent = match (&k.folder, &k.file) {
        (Some(folder), _) => Some(Extent::Folder(folder.clone())),
        (None, Some(file)) => Some(Extent::File(file.clone())),
        (None, None) if !k.citable => Some(Extent::Complement),
        (None, None) => None,
    };
    let form = if k.rules {
        Form::Rule {
            value_chapter: k.value_chapter.clone(),
        }
    } else if k.values {
        Form::Value { chapter: None }
    } else if k.value_chapter.is_some() {
        Form::Value {
            chapter: k.value_chapter.clone(),
        }
    } else {
        Form::Prose
    };
    // §FS-config.3.4: the `index` default is keyed on the name, and this is where a
    // *declared* kind picks it up, after the refusals above read `index` as the
    // file wrote it.
    let index = if k.index == KindIndex::Default && k.folder.is_some() {
        default_kind_index(&k.kind)
    } else {
        k.index.clone()
    };
    Row {
        name: k.kind.clone(),
        places: extent
            .into_iter()
            .map(|extent| Place {
                extent,
                scanned: k.scan,
            })
            .collect(),
        kind: k.citable.then(|| Kind {
            id_format: k.format.clone(),
            form,
            index,
            origin: match &k.fetch {
                Some(fetch) => Origin::External {
                    fetch: fetch.clone(),
                },
                None => Origin::Local,
            },
        }),
    }
}
