//! Discovery and one-read enrollment of a value home's value sources
//! (§FS-values.2.2, §AR-scanner.2.1.3, §AR-scanner.3): one loop reads a source of
//! either format, its format picking only the reader (§FS-values.2.2.4), and both
//! enroll through `value_json_enrollment.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::json::{JsonNode, JsonReader};
use super::scan_error::{
    FileKind, Place, ScanFailure, publish_unlisted, publish_value_source_failure,
};
use super::tree::ScanError;
use super::value_json_enrollment::{enroll_json_member, push_json_invalid};
use super::value_yaml_subset::yaml_value_root;
use super::yaml::parse_yaml;
use crate::config::{Extent, Frame, NamesVerdict, Schema};
use crate::model::sort_path_key;
use crate::model::{
    Catalog, DeclarationSource, ReadingConflict, ReadingMove, ValueFormat, normalize_path_lexically,
};
use crate::sources::io_reason;
use crate::sources::{paths_same_location, physical_path_key};

/// The words a value home's read failure keeps around the shared I/O reason
/// (§FS-check.2.4.1).
const HOME_CONTEXT: &str = "read value home";

/// A value home the enumerator could not list, at open or partway through: the home as
/// the config spells it, and the listing's one error (§AR-scanner.1.relisted).
struct UnreadHome {
    folder: PathBuf,
    error: std::io::Error,
}

impl UnreadHome {
    /// The failure as it is published where nothing else could have reported it, the home's
    /// context around the shared I/O reason (§FS-check.2.4.1).
    fn into_scan_error(self, version: u32) -> ScanError {
        let reason = io_reason(&self.error);
        let message = format!("{HOME_CONTEXT}: {reason}");
        ScanError::new(self.folder, message, home_failure(version))
    }
}

/// §FS-check.2.4.3: a value home is a scan root, so `exclude` does not reach it.
fn home_failure(version: u32) -> ScanFailure {
    ScanFailure::Unread {
        place: Place::Root,
        version,
    }
}

/// Full-catalog enrollment through the same readers fetch uses for a selected
/// home (§FS-values.2.2, §FS-fetch.5.1), retaining readable sources alongside
/// candidate classification failures (§FS-values.5.3.1).
pub(super) fn scan_value_sources(
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
    errors: &mut Vec<ScanError>,
) {
    let sources = match value_sources_in(schema, frame, None, errors) {
        Ok(sources) => sources,
        // §AR-scanner.1.relisted: the walk lists a home too, so its listing failure is
        // published only where the scan has not reported it already (§FS-check.2.4.2).
        Err(UnreadHome { folder, error }) => {
            let failure = home_failure(frame.version);
            publish_unlisted(
                &folder,
                folder.clone(),
                &error,
                Some(HOME_CONTEXT),
                failure,
                errors,
            );
            return;
        }
    };
    scan_sources(schema, frame, sources, findings, errors);
}

/// The catalog's value declarations for one selected folder home and owner
/// (§FS-fetch.5.1). Alias metadata from other homes preserves the catalog's
/// reader and ownership decisions, but unrelated bodies are not read. Content
/// findings stay in the catalog; selected-home discovery/read failures prevent installation.
/// Candidate classification failures have the same scope (§FS-values.5.3.1).
pub(crate) fn folder_value_declarations(
    schema: &Schema,
    frame: Frame<'_>,
    folder: &Path,
    kind: &str,
) -> std::result::Result<Catalog, ScanError> {
    // §FS-fetch.5.1: fetch runs no walk, so a home it cannot read is its failure as it stands.
    let mut errors = Vec::new();
    let sources = value_sources_in(schema, frame, Some((kind, folder)), &mut errors)
        .map_err(|home| home.into_scan_error(frame.version))?;
    let mut findings = Catalog::default();
    scan_sources(schema, frame, sources, &mut findings, &mut errors);
    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }
    Ok(findings)
}

/// One enrollment path for catalog scans and folder-home fetch discovery
/// (§FS-values.2.2, §FS-fetch.5.1), in either format (§FS-values.2.2.4).
fn scan_sources(
    schema: &Schema,
    frame: Frame<'_>,
    sources: Vec<ValueSource<'_>>,
    findings: &mut Catalog,
    errors: &mut Vec<ScanError>,
) {
    // §AR-scanner.1.8: grouped through the run's table, which keeps every spelling.
    let spellings = &frame.run.spellings;
    let mut grouped = BTreeMap::<PathBuf, (Vec<PathBuf>, Vec<&str>)>::new();
    let mut formats = BTreeMap::new();
    for (path, kind, format) in sources {
        formats.insert(path.clone(), format);
        let (paths, owners) = grouped.entry(spellings.resolve(&path)).or_default();
        if !paths.contains(&path) {
            paths.push(path);
        }
        if !owners.contains(&kind) {
            owners.push(kind);
        }
    }
    let mut grouped = grouped.into_iter().collect::<Vec<_>>();
    for (_, (paths, owners)) in &mut grouped {
        paths.sort_by_key(|path| sort_path_key(path));
        owners.sort();
    }
    grouped.sort_by_key(|(_, (paths, _))| sort_path_key(&paths[0]));
    // §FS-values.2.2.4: one path for either format, which picks only the reader.
    for (physical, (paths, owners)) in grouped {
        // A name that is the file's own, alone, has nothing to judge, so a linkless tree pays
        // nothing here. A link alone can have moved (§FS-scan.checks.reading-moved).
        if paths.len() > 1 || paths[0] != physical {
            if paths.len() > 1 {
                spellings.publish_value_names(&physical, &paths);
            }
            match frame.judge_names(schema, &paths) {
                NamesVerdict::Agree => {}
                // §FS-scan.checks.reading-conflict: read under neither name.
                NamesVerdict::Conflict { readings } => {
                    spellings.refuse(&physical);
                    findings.reading_conflicts.push(ReadingConflict {
                        names: paths,
                        readings,
                    });
                    continue;
                }
                // The walk has judged a name it lists too, and a link earns one warning.
                NamesVerdict::Moved { was, now } => {
                    if !findings.reading_moves.iter().any(|m| m.file == paths[0]) {
                        findings.reading_moves.push(ReadingMove {
                            file: paths[0].clone(),
                            was,
                            now,
                        });
                    }
                }
            }
        }
        let path = &paths[0];
        // §AR-scanner.1.8: every name of one file read one way, so the
        // format is the file's and no longer the first spelling's. A file the rule reads as no
        // value source has earned its `reading-moved` above, and the value reader leaves it.
        let Some(format) = formats[path] else {
            continue;
        };
        // §AR-system.2.13: one location, one text, whichever spelling holds it.
        let text = match frame.sources().text(path) {
            Ok(text) => text.into_owned(),
            Err(error) => {
                errors.push(value_source_error(path, format, &error, frame.version));
                continue;
            }
        };
        match read_value_root(&text, format) {
            Ok(root) => enroll_value_root(frame, path, &owners, &text, root, format, findings),
            Err(message) => {
                let failure = ScanFailure::Unparsed(FileKind::Value(format));
                errors.push(ScanError::new(path.clone(), message, failure));
            }
        }
        if owners.len() > 1 {
            duplicate_owned_source(&paths, findings);
        }
    }
}

/// A value source's read or classification failure (§FS-values.5.3.1), with
/// format context around the shared I/O reason (§FS-check.2.4.1).
fn value_source_error(
    path: &Path,
    format: ValueFormat,
    error: &std::io::Error,
    version: u32,
) -> ScanError {
    ScanError::new(
        path.to_path_buf(),
        format!(
            "could not read value {} source: {}",
            format.name(),
            io_reason(error)
        ),
        ScanFailure::Unread {
            place: Place::Value,
            version,
        },
    )
}

/// A source's root by its format's reader, or the line a stream the reader
/// rejects stops the scan on (§FS-values.2.2.4, §FS-values.5.3).
fn read_value_root(text: &str, format: ValueFormat) -> std::result::Result<JsonNode, String> {
    match format {
        ValueFormat::Json => {
            JsonReader::parse(text).map_err(|message| format!("invalid JSON: {message}"))
        }
        ValueFormat::Yaml => match parse_yaml(text) {
            Ok(stream) => Ok(yaml_value_root(text, &stream)),
            Err(error) => Err(format!(
                "invalid YAML: {}:{}: {}",
                error.line, error.column, error.message
            )),
        },
    }
}

/// A source owned by two homes is ambiguous even after one read, so each other spelling holds its
/// own copy and the ordinary duplicate rule stays the single winner, naming both
/// (§FS-values.2.3, §FS-declarations.checks.duplicate). One name in two homes copies at that name.
fn duplicate_owned_source(paths: &[PathBuf], findings: &mut Catalog) {
    let first = &paths[0];
    let others = if paths.len() > 1 {
        &paths[1..]
    } else {
        std::slice::from_ref(first)
    };
    let duplicated = findings
        .declarations
        .iter()
        .flat_map(|(id, declarations)| {
            declarations
                .iter()
                .filter(|declaration| {
                    matches!(declaration.source, DeclarationSource::Json { .. })
                        && paths_same_location(&declaration.file, first)
                })
                .cloned()
                .map(|declaration| (id.clone(), declaration))
        })
        .collect::<Vec<_>>();
    for (id, declaration) in duplicated {
        let copies = findings.declarations.entry(id).or_default();
        for other in others {
            let mut copy = declaration.clone();
            copy.file = other.clone();
            copies.push(copy);
        }
    }
}

/// One name a value home takes, its owning kind, and the format the rule reads the file in: `None`
/// where its target's own name reads it as no value source (§AR-scanner.1.8).
pub(super) type ValueSource<'a> = (PathBuf, &'a str, Option<ValueFormat>);

/// Every value source of every value home, with its owning kind and format
/// (§FS-values.terms, §FS-values.2.2), in one bytewise path order across formats;
/// the scoped fetch query shares this enumerator (§FS-fetch.5.1).
/// A home's listing failure, at open or partway through, retains home context around
/// the shared I/O reason (§FS-check.2.4.1).
pub(super) fn value_sources<'a>(
    schema: &'a Schema,
    frame: Frame<'a>,
) -> std::result::Result<Vec<ValueSource<'a>>, ScanError> {
    // §FS-values.5.3.1: enrollment publishes classification failures; callers here
    // need the readable source paths for the ordinary walk's source exclusions.
    value_sources_in(schema, frame, None, &mut Vec::new())
        .map_err(|home| home.into_scan_error(frame.version))
}

/// The single source enumerator (§FS-values.2.2, §FS-fetch.5.1). A selected home
/// retains all spellings and owners of its physical inputs before shared grouping
/// chooses a reader. Failures discovering unrelated homes do not gate fetch.
/// Failed candidate metadata is reported while readable sources remain available
/// (§FS-values.5.3.1); confirmed non-files stay outside the source set.
fn value_sources_in<'a>(
    schema: &'a Schema,
    frame: Frame<'a>,
    selected: Option<(&str, &Path)>,
    errors: &mut Vec<ScanError>,
) -> std::result::Result<Vec<ValueSource<'a>>, UnreadHome> {
    let mut sources = Vec::new();
    let mut selected_keys = BTreeSet::new();
    let mut failed_sources = BTreeSet::new();
    // §FS-config-v2.schema.sources.values: the home picks the files, their language the reader.
    let format_of = |path: &Path| {
        frame
            .enters_as_value(schema, path)
            .map(|_| frame.reading(schema, path).value_format())
    };
    for row in schema.value_rows() {
        let kind = row.name.as_str();
        // §FS-config-v2.checks.config-value-home-absent: a home after the row's first that does
        // not exist loaded with a warning, and is not read.
        let absent_later = |home: &str, error: Option<&std::io::Error>| {
            let first = row.places.first().is_some_and(|place| match &place.extent {
                Extent::File(path) | Extent::Folder(path) => path == home,
                Extent::Complement => false,
            });
            !first && error.is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        };
        // §FS-config-v2.schema.places: every file and folder of the row is a value home.
        for file in row.files() {
            let path = normalize_path_lexically(&frame.root().join(file));
            if absent_later(file, frame.sources().try_is_file(&path).err().as_ref()) {
                continue;
            }
            if let Some(format) = format_of(&path) {
                sources.push((path, kind, format));
            }
        }
        for home in row.folders() {
            let folder = normalize_path_lexically(&frame.root().join(home));
            let selected_home = selected
                .is_some_and(|(owner, home)| owner == kind && paths_same_location(home, &folder));
            let required = selected.is_none() || selected_home;
            // §FS-check.6.1.1: cover this effective input before its shared read.
            let disk = frame.sources();
            let listing = disk.list_dir(&folder);
            if absent_later(home, listing.failure.as_ref()) {
                continue;
            }
            if required && let Some(error) = listing.failure {
                return Err(UnreadHome { folder, error });
            }
            // §FS-values.5.3.1: classification failures have a deterministic path order too.
            let mut entries = listing.entries;
            entries.sort_by_key(|path| sort_path_key(path));
            // §FS-fetch.5.1: a home fetch does not select is read as far as it lists.
            for listed in entries {
                let path = normalize_path_lexically(&listed);
                if let Some(spelled) = frame.enters_as_value(schema, &path) {
                    match disk.try_is_file(&path) {
                        Ok(true) => {}
                        Ok(false) => continue,
                        Err(error) => {
                            // §FS-values.5.3.1: one failed candidate, even if two homes own it;
                            // unrelated homes do not gate selected fetch. Named as it entered,
                            // whatever its target reads as.
                            if required && failed_sources.insert(physical_path_key(&path)) {
                                let report =
                                    value_source_error(&path, spelled, &error, frame.version);
                                publish_value_source_failure(report, &error, errors);
                            }
                            continue;
                        }
                    }
                    let format = frame.reading(schema, &path).value_format();
                    if selected_home {
                        selected_keys.insert(physical_path_key(&path));
                    }
                    sources.push((path, kind, format));
                }
            }
            for path in disk.buffer_only_files() {
                if path
                    .parent()
                    .is_some_and(|parent| paths_same_location(parent, &folder))
                    && let Some(format) = format_of(&path)
                {
                    if selected_home {
                        selected_keys.insert(physical_path_key(&path));
                    }
                    sources.push((path, kind, format));
                }
            }
        }
    }
    if selected.is_some() {
        // §FS-fetch.5.1: filter physical groups, never the aliases deciding their recognition.
        sources.retain(|(path, _, _)| selected_keys.contains(&physical_path_key(path)));
    }
    sources.sort_by(|(left, left_kind, _), (right, right_kind, _)| {
        (sort_path_key(left), left_kind).cmp(&(sort_path_key(right), right_kind))
    });
    sources.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);
    Ok(sources)
}

/// The noun with its indefinite article, for a reason naming a format's word.
pub(super) fn a(noun: &str) -> String {
    let article = if noun.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {noun}")
}

/// Enroll one source's root, read by either reader (§FS-values.2.2.1).
fn enroll_value_root(
    frame: Frame<'_>,
    path: &Path,
    owners: &[&str],
    text: &str,
    root: JsonNode,
    format: ValueFormat,
    findings: &mut Catalog,
) {
    let (name, (object, _)) = (format.name(), format.collections());
    let (members, span) = match root {
        JsonNode::Object(members, span) => (members, span),
        // §FS-values.2.2.3: a construct the YAML subset refuses, named.
        JsonNode::Refused(span, reason) => {
            push_json_invalid(findings, None, path, text, span, &reason, format);
            return;
        }
        root => {
            let message = format!("value {name} root must be {}", a(object));
            push_json_invalid(findings, None, path, text, root.span(), &message, format);
            return;
        }
    };
    if members.is_empty() {
        let message = format!("value {name} {object} must not be empty");
        push_json_invalid(findings, None, path, text, span, &message, format);
    }
    for member in members {
        enroll_json_member(frame, path, owners, text, member, format, findings);
    }
}
