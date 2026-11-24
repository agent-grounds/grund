// The installed snapshot and how a rebuild makes the next one (§AR-lsp.2.1): the projects an edit
// covers rebuilt from the documents a rebuild was handed, the rest carried over, shared rather than
// copied. The worker runs it off the event loop (§AR-lsp.2.1.1).

impl Installed {
    fn new(definition_link_support: bool) -> Self {
        Self {
            folder_roots: BTreeMap::new(),
            folder_config_failures: BTreeMap::new(),
            scan_config_failures: BTreeMap::new(),
            projects: Vec::new(),
            imports: Vec::new(),
            docs: BTreeMap::new(),
            versions: BTreeMap::new(),
            definition_link_support,
            line_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// What a rebuild starts from: the projects shared rather than copied, and neither the
    /// documents, which the rebuild's inputs replace, nor the line cache, which it clears.
    fn base(&self) -> Self {
        Self {
            folder_roots: self.folder_roots.clone(),
            folder_config_failures: self.folder_config_failures.clone(),
            scan_config_failures: self.scan_config_failures.clone(),
            projects: self.projects.clone(),
            imports: self.imports.clone(),
            docs: BTreeMap::new(),
            versions: BTreeMap::new(),
            definition_link_support: self.definition_link_support,
            line_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Settle every grammar a keystroke reaches in this snapshot, on the calling thread: the event
    /// loop's, which a cache taken on the worker would not serve (§AR-lsp.2.1.1). A project
    /// carried over from the last snapshot is settled already, and costs one look per pattern.
    fn settle(&self) {
        for project in self.projects.iter().chain(&self.imports) {
            if let Some(completion) = &project.completion {
                completion.settle();
            }
        }
    }

    /// Rebuild the projects `scope` covers from `folders` and this snapshot's documents, and
    /// carry the rest over untouched (§AR-lsp.2.1). A keystroke in one editor folder cannot change
    /// the verdicts of a project that never reads that file, so those are left standing. A path
    /// *no* project claims widens the rebuild to everything: it is the shape a newly created file
    /// takes under a symlinked or parent-relative include root, outside every project root and in
    /// no scan yet, and guessing "nobody's" there would leave the editor silent on a file the CLI
    /// checks.
    ///
    /// One failing project never takes the others with it. A folder whose config will not load,
    /// or whose scan errors out, is noted on stderr and keeps whatever snapshot it last had — so a
    /// half-typed `grund.toml` in one editor folder cannot freeze diagnostics across the whole
    /// session (§FS-lsp.2.2.4, §REQ-never-crashes). A config that will not load, at discovery or in
    /// the scan, is also recorded to be published on that config (§FS-lsp.2.2.4). The worker
    /// runs it, off the event loop (§AR-lsp.2.1.1); only the session's first snapshot is built
    /// on the calling thread, by `refresh`.
    fn rebuild(&mut self, folders: &BTreeMap<Url, PathBuf>, scope: &Scope) {
        // A folder the client removed after this snapshot was installed is forgotten here.
        self.folder_roots.retain(|uri, _| folders.contains_key(uri));
        let changed: Option<Vec<PathBuf>> = match scope {
            Scope::All => None,
            Scope::Paths(paths) => {
                let paths: Vec<PathBuf> = paths.iter().map(normalize_path).collect();
                let claimed = |path: &PathBuf| {
                    self.projects
                        .iter()
                        .chain(&self.imports)
                        .any(|project| project.might_cover(path))
                };
                paths.iter().all(claimed).then_some(paths)
            }
        };
        let mut roots = BTreeSet::new();
        let mut discoverable_roots = BTreeSet::new();
        self.folder_config_failures.clear();
        for (folder_uri, folder) in folders {
            match project_root(folder_uri, folder) {
                Ok(root) => {
                    self.folder_roots.insert(folder_uri.clone(), root.clone());
                    discoverable_roots.insert(root.clone());
                    roots.insert(root);
                }
                Err(err) => {
                    eprintln!(
                        "grund-lsp: workspace folder `{}` has no usable config: {err:#}",
                        folder.display()
                    );
                    let previous_root = self.folder_roots.get(folder_uri).cloned();
                    if let Some(failure) = ConfigFailure::of(&err, previous_root.clone()) {
                        self.folder_config_failures
                            .insert(folder_uri.clone(), failure);
                    }
                    roots.extend(previous_root);
                }
            }
        }
        let overlays = self.open_document_overlays();
        let mut previous: BTreeMap<PathBuf, Arc<ProjectSnapshot>> =
            std::mem::take(&mut self.projects)
                .into_iter()
                .map(|project| (project.root.clone(), project))
                .collect();
        let mut projects = Vec::new();
        // A root no folder discovers any more has no scan of its own to fail:
        // what is wrong with it is the folder's failure, recorded above.
        self.scan_config_failures
            .retain(|root, _| discoverable_roots.contains(root));
        for root in roots {
            let existing = previous.remove(&root);
            // Discovery itself failed for every anchor that last named this
            // root. Calling `lsp_snapshot` would rediscover the same broken
            // config, so retain the existing snapshot directly (§FS-lsp.2.2.4).
            if !discoverable_roots.contains(&root) {
                projects.extend(existing);
                continue;
            }
            let unaffected = match (&changed, &existing) {
                (Some(paths), Some(project)) => !paths.iter().any(|path| project.might_cover(path)),
                _ => false,
            };
            if unaffected {
                projects.extend(existing);
                continue;
            }
            match lsp_snapshot_with_completion(LspSnapshotOpts {
                path: root.clone(),
                // A discovered project root is the whole project, so its
                // configured include paths govern just as for a root CLI run.
                path_provided: true,
                open_documents: overlays.clone(),
            }) {
                Ok(snapshot) => {
                    self.scan_config_failures.remove(&root);
                    projects.push(Arc::new(ProjectSnapshot::with_completion(root, snapshot)));
                }
                Err(err) => {
                    eprintln!("grund-lsp: scan of `{}` failed: {err:#}", root.display());
                    // §FS-lsp.2.2.4: a member's config, say, that fails in the scan.
                    let retained = existing.as_ref().map(|project| project.root.clone());
                    match ConfigFailure::of(&err, retained) {
                        Some(failure) => self.scan_config_failures.insert(root, failure),
                        None => self.scan_config_failures.remove(&root),
                    };
                    projects.extend(existing);
                }
            }
        }
        self.projects = projects;
        self.rebuild_imports(changed.as_deref(), &overlays);
        // The rebuilt snapshot reflects the new document contents, so any cached
        // lines from before the edit are stale.
        self.line_cache.borrow_mut().clear();
    }
}
