// The rebuild worker and the queue in front of it (§AR-lsp.2.1.1, §AR-lsp.2.1.2): one thread off
// the loop runs every rebuild, edits that arrive meanwhile merge into the one still to start, and
// the loop installs each finished rebuild between two messages (§AR-lsp.2.1.3).

/// What a rebuild covers: every project, or the ones that cover one of the edited paths.
#[derive(Clone, Debug)]
enum Scope {
    All,
    Paths(BTreeSet<PathBuf>),
}

impl Scope {
    fn add(&mut self, other: Scope) {
        match (&mut *self, other) {
            (Scope::Paths(paths), Scope::Paths(more)) => paths.extend(more),
            (_, _) => *self = Scope::All,
        }
    }
}

/// What a rebuild reads besides the snapshot it starts from: the folders, and the documents with
/// their versions, as they stood at the edit it ends at.
struct Inputs {
    folders: BTreeMap<Url, PathBuf>,
    docs: BTreeMap<Url, String>,
    versions: BTreeMap<Url, i32>,
}

/// A rebuild the worker has not started: every edit up to `seq` that no earlier one takes.
struct Pending {
    seq: u64,
    scope: Scope,
    /// The held requests that wait on exactly this rebuild. A rebuild one waits on is sealed: it
    /// takes no later edit, so the request is answered against the text it was sent with
    /// (§AR-lsp.2.1.2).
    seals: usize,
    /// Taken when the rebuild is sealed, else when it starts, which is then its own `seq`.
    inputs: Option<Inputs>,
}

/// What the loop records as a message arrives, for `dispatch` to apply before the next one,
/// outside the counted keystroke (§AR-lsp.2.1.1).
enum Recorded {
    /// An edit, merged into the rebuild still to start.
    Edit { seq: u64, scope: Scope },
    /// A request held on the rebuild ending at this edit, which it seals.
    Seal(u64),
}

/// One rebuild handed to the worker.
struct Job {
    seq: u64,
    scope: Scope,
    inputs: Inputs,
    state: Installed,
    /// A case's ask that this rebuild panic, as a defect in the scan would make it.
    #[cfg(test)]
    panics: bool,
}

/// A finished rebuild: the snapshot with the diagnostics computed over it, or nothing where the
/// rebuild panicked, which leaves the last snapshot standing and its scope to rebuild again
/// (§AR-lsp.2.1.1).
struct Built {
    seq: u64,
    scope: Scope,
    outcome: Option<(Installed, BTreeMap<Url, Vec<Diagnostic>>)>,
}

impl Job {
    /// §AR-lsp.2.1.1: the rebuild as the loop ran it before, and the diagnostics over it.
    fn run(self) -> Built {
        let Job {
            seq,
            scope,
            inputs,
            mut state,
            #[cfg(test)]
            panics,
        } = self;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            #[cfg(test)]
            assert!(!panics, "a rebuild panics, as the case asked");
            state.docs = inputs.docs;
            state.versions = inputs.versions;
            state.rebuild(&inputs.folders, &scope);
            let diagnostics = state.diagnostics();
            (state, diagnostics)
        }));
        if outcome.is_err() {
            eprintln!("grund-lsp: a rebuild panicked; the last snapshot stands");
        }
        Built {
            seq,
            scope,
            outcome: outcome.ok(),
        }
    }
}

/// The worker's loop: one rebuild at a time, in the order the loop hands them over, on the Rayon
/// pool as `RAYON_NUM_THREADS` pins it, until the server drops its end (§AR-lsp.2.1.1).
fn work(
    jobs: crossbeam_channel::Receiver<Job>,
    built: crossbeam_channel::Sender<Built>,
    #[cfg(test)] gate: Option<Arc<RebuildGate>>,
) {
    for job in jobs {
        #[cfg(test)]
        let seq = job.seq;
        #[cfg(test)]
        let mut job = job;
        #[cfg(test)]
        if let Some(gate) = &gate {
            job.panics = gate.enter(&job);
        }
        let outcome = job.run();
        // Told before the result goes, so a case that saw the result install has the event too.
        #[cfg(test)]
        if let Some(gate) = &gate {
            gate.leave(seq);
        }
        if built.send(outcome).is_err() {
            return;
        }
    }
}

/// How the loop stands with its rebuilds (§AR-lsp.2.1.2).
struct Rebuilds {
    /// The client's edits, counted: every message that can change what a rebuild reads.
    seq: u64,
    /// The edit the installed snapshot reflects.
    installed: u64,
    /// The edit the worker's rebuild ends at, while it runs.
    in_flight: Option<u64>,
    queue: VecDeque<Pending>,
    /// What the loop recorded since `dispatch` last brought the queue up to date, in arrival
    /// order.
    recorded: Vec<Recorded>,
    /// Requests that wait on a rebuild, in arrival order.
    held: Vec<Held>,
    /// Every document edited this session, with its latest edit and how that text stands against
    /// the installed catalog (§FS-lsp.responsiveness.4).
    edited: BTreeMap<Url, Edited>,
    /// The latest edit that can change any project — a folder or watched-file change, a disk event
    /// — while no snapshot reflects it yet.
    stale_all: Option<u64>,
    /// What rebuilds that panicked covered while no rebuild was queued to take it: the next
    /// rebuild to start rebuilds it again (§AR-lsp.2.1.1).
    failed: Option<Scope>,
    /// The client sent `exit` while its `shutdown` was held (§AR-lsp.2.1.2).
    exited: bool,
    /// The worker, spawned by the first rebuild it is handed.
    jobs: Option<crossbeam_channel::Sender<Job>>,
    built: (
        crossbeam_channel::Sender<Built>,
        crossbeam_channel::Receiver<Built>,
    ),
    #[cfg(test)]
    gate: Option<Arc<RebuildGate>>,
}

impl Default for Rebuilds {
    fn default() -> Self {
        Self {
            seq: 0,
            installed: 0,
            in_flight: None,
            queue: VecDeque::new(),
            recorded: Vec::new(),
            held: Vec::new(),
            edited: BTreeMap::new(),
            stale_all: None,
            failed: None,
            exited: false,
            jobs: None,
            built: crossbeam_channel::unbounded(),
            #[cfg(test)]
            gate: None,
        }
    }
}

impl Rebuilds {
    /// Count one edit and record it for `dispatch`, which merges it into the queue.
    fn record(&mut self, scope: Scope) -> u64 {
        self.seq += 1;
        self.recorded.push(Recorded::Edit {
            seq: self.seq,
            scope,
        });
        self.seq
    }

    /// Merge the edit at `seq` into the last rebuild still to start where no request seals it,
    /// with whatever a rebuild that panicked left to rebuild again.
    fn enqueue(&mut self, seq: u64, mut scope: Scope) {
        if let Some(failed) = self.failed.take() {
            scope.add(failed);
        }
        match self.queue.back_mut() {
            Some(last) if last.seals == 0 => {
                last.seq = seq;
                last.scope.add(scope);
            }
            _ => self.queue.push_back(Pending {
                seq,
                scope,
                seals: 0,
                inputs: None,
            }),
        }
    }

    /// A rebuild over `scope` panicked: the next rebuild to start rebuilds it again, the one
    /// queued or else the next an edit or a held typing request asks for (§AR-lsp.2.1.1).
    fn fail(&mut self, scope: Scope) {
        match (self.queue.front_mut(), &mut self.failed) {
            (Some(next), _) => next.scope.add(scope),
            (None, Some(failed)) => failed.add(scope),
            (None, None) => self.failed = Some(scope),
        }
    }

    /// The installed snapshot reflects every edit counted so far, and no rebuild is still to run.
    fn current(&self) -> bool {
        self.installed == self.seq && self.in_flight.is_none() && self.queue.is_empty()
    }

    /// A request held for the rebuild ending at `seq` has gone: that rebuild takes later edits
    /// again once nothing else waits on it, merged into the next where one is queued.
    fn unseal(&mut self, seq: u64) {
        let Some(index) = self.queue.iter().position(|pending| pending.seq == seq) else {
            return;
        };
        let pending = &mut self.queue[index];
        pending.seals = pending.seals.saturating_sub(1);
        if pending.seals > 0 {
            return;
        }
        if index + 1 < self.queue.len() {
            let freed = self.queue.remove(index).expect("the index is in the queue");
            self.queue[index].scope.add(freed.scope);
        } else {
            self.queue[index].inputs = None;
        }
    }
}

impl Server {
    /// The inputs of a rebuild that ends at the latest edit.
    fn inputs(&self) -> Inputs {
        Inputs {
            folders: self.workspace_folders.clone(),
            docs: self.open_docs.clone(),
            versions: self.versions.clone(),
        }
    }

    /// Bring the queue up to date with what the loop recorded, in arrival order: each edit merged
    /// into the rebuild still to start, each seal applied (§AR-lsp.2.1.2). Between two messages,
    /// so the documents a seal takes are the ones its request was sent against.
    fn fold(&mut self) {
        let mut recorded = std::mem::take(&mut self.rebuilds.recorded);
        for item in recorded.drain(..) {
            match item {
                Recorded::Edit { seq, scope } => self.rebuilds.enqueue(seq, scope),
                Recorded::Seal(seq) => self.seal(seq),
            }
        }
        self.rebuilds.recorded = recorded;
    }

    /// Seal the rebuild that ends at `seq` for a request held on it, taking its inputs now. Where
    /// none is outstanding, the request is a typing request held by edits a rebuild that panicked
    /// left unreflected, and that rebuild runs again for it (§AR-lsp.2.1.1).
    fn seal(&mut self, seq: u64) {
        if self.rebuilds.in_flight == Some(seq) {
            return;
        }
        if !matches!(self.rebuilds.queue.back(), Some(last) if last.seq == seq) {
            let scope = self.rebuilds.failed.take().unwrap_or(Scope::All);
            self.rebuilds.queue.push_back(Pending {
                seq,
                scope,
                seals: 0,
                inputs: None,
            });
        }
        let unread = matches!(self.rebuilds.queue.back(),
            Some(last) if last.seq == seq && last.inputs.is_none());
        let inputs = unread.then(|| self.inputs());
        if let Some(last) = self.rebuilds.queue.back_mut()
            && last.seq == seq
        {
            last.seals += 1;
            if let Some(inputs) = inputs {
                last.inputs = Some(inputs);
            }
        }
    }

    /// Hand the next rebuild to the worker when it is idle (§AR-lsp.2.1.1), once the queue holds
    /// what the loop recorded. Outside the counted keystroke: whether it runs depends on the
    /// worker, and a keystroke's cost must not (R3).
    fn dispatch(&mut self) {
        self.fold();
        if self.rebuilds.in_flight.is_some() {
            return;
        }
        let Some(pending) = self.rebuilds.queue.pop_front() else {
            return;
        };
        let inputs = pending.inputs.unwrap_or_else(|| self.inputs());
        let job = Job {
            seq: pending.seq,
            scope: pending.scope,
            inputs,
            state: self.installed.base(),
            #[cfg(test)]
            panics: false,
        };
        self.rebuilds.in_flight = Some(pending.seq);
        if self.rebuilds.jobs.is_none() {
            self.rebuilds.jobs = self.spawn_worker();
        }
        let refused = match &self.rebuilds.jobs {
            Some(jobs) => jobs.send(job).err().map(|refused| refused.0),
            None => Some(job),
        };
        // A worker that cannot start, or has gone, leaves the rebuild to this thread.
        if let Some(job) = refused {
            self.rebuilds.jobs = None;
            let _ = self.rebuilds.built.0.send(job.run());
        }
    }

    /// Run every rebuild still to run and install it, as the loop does between messages, until
    /// none is in flight or one ends the session (§AR-lsp.2.1.2).
    fn drain(&mut self) -> Result<Flow> {
        loop {
            self.dispatch();
            if self.rebuilds.in_flight.is_none() {
                return Ok(Flow::Continue);
            }
            let built = self.rebuilds.built.1.recv()?;
            if self.install(built)? == Flow::Exit {
                return Ok(Flow::Exit);
            }
        }
    }

    fn spawn_worker(&self) -> Option<crossbeam_channel::Sender<Job>> {
        let (jobs, inbox) = crossbeam_channel::unbounded();
        let built = self.rebuilds.built.0.clone();
        #[cfg(test)]
        let gate = self.rebuilds.gate.clone();
        let spawned = std::thread::Builder::new()
            .name("grund-lsp-rebuild".into())
            .spawn(move || {
                work(
                    inbox,
                    built,
                    #[cfg(test)]
                    gate,
                )
            });
        match spawned {
            Ok(_) => Some(jobs),
            Err(err) => {
                eprintln!("grund-lsp: the rebuild worker did not start: {err}");
                None
            }
        }
    }

    /// Install a finished rebuild between two messages (§AR-lsp.2.1.3): swap the snapshot, judge
    /// the edits it does not reflect again against it (§FS-lsp.responsiveness.4), publish with
    /// the versions it checked (§FS-lsp.1.1), then answer what waited on it, in arrival order. A
    /// rebuild that panicked swaps nothing in: the edits it covered stay as they were judged, and
    /// its scope waits for the next rebuild (§AR-lsp.2.1.1).
    fn install(&mut self, built: Built) -> Result<Flow> {
        self.fold();
        let Built {
            seq,
            scope,
            outcome,
        } = built;
        self.rebuilds.in_flight = None;
        self.rebuilds.installed = seq;
        let rebuilt = outcome.is_some();
        match outcome {
            Some((state, diagnostics)) => {
                let replaced = std::mem::replace(&mut self.installed, state);
                drop(replaced);
                self.watch_imports();
                self.judge_again(seq);
                if self.rebuilds.stale_all.is_some_and(|stale| stale <= seq) {
                    self.rebuilds.stale_all = None;
                }
                self.publish(diagnostics)?;
            }
            None => self.rebuilds.fail(scope),
        }
        let (due, waiting) = std::mem::take(&mut self.rebuilds.held)
            .into_iter()
            .partition::<Vec<_>, _>(|held| held.seq <= seq);
        self.rebuilds.held = waiting;
        for held in due {
            if self.answer_held(held, rebuilt)? == Flow::Exit {
                return Ok(Flow::Exit);
            }
        }
        Ok(Flow::Continue)
    }
}
