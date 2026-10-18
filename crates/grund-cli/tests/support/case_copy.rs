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
            (resolved_repo.as_path(), resolved_root.join(&logical)),
        ] {
            let actual = actual.to_string_lossy().into_owned();
            let stable = stable.to_string_lossy().into_owned();
            self.output_prefixes.push((actual.clone(), stable.clone()));
            // Windows APIs may return a verbatim prefix, or forward slashes.
            let actual = actual.strip_prefix(r"\\?\").unwrap_or(&actual);
            let stable = stable.strip_prefix(r"\\?\").unwrap_or(&stable);
            self.output_prefixes.push((actual.into(), stable.into()));
            self.output_prefixes
                .push((actual.replace('\\', "/"), stable.replace('\\', "/")));
        }
        self.output_prefixes
            .sort_by_key(|(actual, _)| std::cmp::Reverse(actual.len()));
        self.output_prefixes.dedup();
    }

    /// Both compare and refresh receive the same output (§FS-examples.5.6.2).
    fn normalize(&self, text: &str) -> String {
        let mut text = text.to_string();
        for (actual, logical) in &self.output_prefixes {
            text = replace_copy_prefix(&text, actual, logical);
        }
        text
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

/// Replace a directory prefix, never a different path containing that text
/// (§FS-examples.5.6.2). A suffix separator belongs to a path inside the copy;
/// an adjacent filename character instead belongs to another directory's name.
fn replace_copy_prefix(text: &str, actual: &str, logical: &str) -> String {
    let path_character =
        |c: char| c.is_alphanumeric() || matches!(c, '/' | '\\' | '.' | '_' | '-' | ':');
    let mut result = String::with_capacity(text.len());
    let mut copied = 0;
    for (start, matched) in text.match_indices(actual) {
        let end = start + matched.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        if before.is_some_and(path_character)
            || after.is_some_and(|c| path_character(c) && c != '/' && c != '\\')
        {
            continue;
        }
        result.push_str(&text[copied..start]);
        result.push_str(logical);
        copied = end;
    }
    result.push_str(&text[copied..]);
    result
}
