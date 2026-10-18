// §FS-examples.5.6.1: one allocation owns every use, including preparation and unwind.
struct CaseCopy {
    scratch: PathBuf,
    repo: PathBuf,
    output_prefixes: Vec<(String, String)>,
}

impl CaseCopy {
    fn new(manifest_dir: &Path, name: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let work = manifest_dir.join("target/e2e-work");
        fs::create_dir_all(&work)
            .unwrap_or_else(|err| panic!("{name}: create {}: {err}", work.display()));
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        for _ in 0..128 {
            let scratch = work.join(format!(
                "invocation-{}-{stamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&scratch) {
                Ok(()) => {
                    // Install the guard before any further fallible preparation.
                    let mut copy = Self {
                        repo: scratch.join("repo"),
                        scratch,
                        output_prefixes: Vec::new(),
                    };
                    copy.set_output_prefixes(manifest_dir, name);
                    return copy;
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => panic!("{name}: create {}: {err}", scratch.display()),
            }
        }
        panic!("{name}: could not allocate an invocation directory after 128 attempts");
    }

    /// §FS-examples.5.6.2: record exact argv and resolved absolute spellings once.
    fn set_output_prefixes(&mut self, manifest_dir: &Path, name: &str) {
        let logical = case_paths::logical_copy(name);
        let relative = self.repo.strip_prefix(manifest_dir).unwrap_or(&self.repo);
        let resolved_root = fs::canonicalize(manifest_dir)
            .unwrap_or_else(|err| panic!("{name}: resolve {}: {err}", manifest_dir.display()));
        let resolved_repo = resolved_root.join(relative);
        for (actual, stable) in [
            (relative, logical.clone()),
            (self.repo.as_path(), manifest_dir.join(&logical)),
            // Canonical cwd output still uses the authored absolute logical
            // spelling, including any mixed separators in the manifest root.
            (resolved_repo.as_path(), manifest_dir.join(&logical)),
        ] {
            let actual = actual.to_string_lossy().into_owned();
            let stable = stable.to_string_lossy().into_owned();
            self.output_prefixes
                .extend(copy_path_spellings(&actual, &stable, cfg!(windows)));
        }
        self.output_prefixes
            .sort_by_key(|(actual, _)| std::cmp::Reverse(actual.len()));
        self.output_prefixes.dedup();
    }

    /// Both compare and refresh receive the same output (§FS-examples.5.6.2).
    fn normalize(&self, text: &str) -> String {
        normalize_copy_output(text, &self.output_prefixes, cfg!(windows))
    }
}

impl Drop for CaseCopy {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.scratch)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            let message = format!("remove invocation {}: {err}", self.scratch.display());
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// Windows path producers can use argv's mixed separators, native separators,
/// forward slashes or a verbatim prefix (§FS-examples.5.6.2). Native output maps
/// to the authored PathBuf spelling; explicitly slash-rendered output keeps its
/// slash-rendered golden. Unix backslashes are filename bytes, not separators.
fn copy_path_spellings(actual: &str, logical: &str, windows: bool) -> Vec<(String, String)> {
    let mut spellings = vec![(actual.into(), logical.into())];
    if windows {
        let actual = actual.strip_prefix(r"\\?\").unwrap_or(actual);
        let logical = logical.strip_prefix(r"\\?\").unwrap_or(logical);
        spellings.extend([
            (actual.into(), logical.into()),
            (actual.replace('/', "\\"), logical.into()),
            (actual.replace('\\', "/"), logical.replace('\\', "/")),
        ]);
    }
    spellings
}

/// Output paths are whole whitespace-delimited fields or quoted fields, as in
/// diagnostics and JSON (§FS-examples.5.6.2). Parse those contexts instead of
/// guessing which punctuation is legal in a filename. A field must be the owned
/// path or its descendant; sibling suffixes and enclosing names stay significant.
/// Quoted fields include their spaces. Quotes embedded in a bare filename or
/// prose (such as a contraction) do not open a new field.
fn normalize_copy_output(text: &str, prefixes: &[(String, String)], windows: bool) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    let quote = |c| matches!(c, '\'' | '"' | '`');
    let mut bare_start = true;
    while let Some(first) = rest.chars().next() {
        if first.is_whitespace() {
            result.push(first);
            rest = &rest[first.len_utf8()..];
            bare_start = true;
            continue;
        }
        let quoted_start =
            bare_start || text[..text.len() - rest.len()].ends_with([':', '=', '[', '(', ',']);
        if quote(first)
            && (!quoted_start
                || !rest[first.len_utf8()..]
                    .find([first, '\n'])
                    .is_some_and(|end| rest[first.len_utf8() + end..].starts_with(first)))
        {
            result.push(first);
            rest = &rest[first.len_utf8()..];
            bare_start = false;
            continue;
        }
        let (field, tail, closing) = if quote(first) {
            result.push(first);
            rest = &rest[first.len_utf8()..];
            let Some(end) = rest.find(first) else {
                result.push_str(rest);
                break;
            };
            (&rest[..end], &rest[end + first.len_utf8()..], Some(first))
        } else {
            // A known absolute prefix may itself contain spaces. Only scan for
            // the field's end after that exact prefix, rather than splitting it.
            let prefix_len = prefixes
                .iter()
                .find(|(actual, _)| rest.starts_with(actual))
                .map_or(0, |(actual, _)| actual.len());
            let end = prefix_len
                + rest[prefix_len..]
                    .find(|c: char| c.is_whitespace() || quote(c))
                    .unwrap_or(rest.len() - prefix_len);
            (&rest[..end], &rest[end..], None)
        };
        let eligible = if closing.is_some() {
            quoted_start
                && tail
                    .chars()
                    .next()
                    .is_none_or(|c| c.is_whitespace() || matches!(c, ':' | ',' | ']' | '}' | ')'))
        } else {
            bare_start && !tail.starts_with(quote)
        };
        let replacement = eligible
            .then(|| {
                prefixes.iter().find_map(|(actual, logical)| {
                    let suffix = field.strip_prefix(actual)?;
                    (suffix.is_empty()
                        || suffix.starts_with('/')
                        || (windows && suffix.starts_with('\\')))
                    .then_some((logical, suffix))
                })
            })
            .flatten();
        if let Some((logical, suffix)) = replacement {
            result.push_str(logical);
            result.push_str(suffix);
        } else {
            result.push_str(field);
        }
        if let Some(closing) = closing {
            result.push(closing);
        }
        rest = tail;
        bare_start = false;
    }
    result
}
