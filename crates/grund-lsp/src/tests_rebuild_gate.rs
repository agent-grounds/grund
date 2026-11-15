// The test seam over the rebuild worker (§AR-lsp.2.1.2): a gate a case holds closed so that no
// rebuild finishes until it says so, the rebuilds it counts as they start, and the synchronous
// handlers the older cases drive the server through.

/// What a case sees of the worker: each rebuild as it starts, with the documents it reads, and
/// each as it finishes.
#[derive(Debug)]
enum Rebuilding {
    Started {
        seq: u64,
        docs: BTreeMap<Url, String>,
    },
    Finished {
        seq: u64,
    },
}

/// Permits for rebuilds to run: `None` lets every one through.
struct RebuildGate {
    permits: std::sync::Mutex<Option<usize>>,
    turn: std::sync::Condvar,
    events: crossbeam_channel::Sender<Rebuilding>,
}

impl RebuildGate {
    /// Announce `job` and wait for a permit to run it.
    fn enter(&self, job: &Job) {
        let started = Rebuilding::Started {
            seq: job.seq,
            docs: job.inputs.docs.clone(),
        };
        let _ = self.events.send(started);
        let mut permits = self.permits.lock().unwrap();
        loop {
            match permits.as_mut() {
                None => return,
                Some(left) if *left > 0 => {
                    *left -= 1;
                    return;
                }
                Some(_) => permits = self.turn.wait(permits).unwrap(),
            }
        }
    }

    fn leave(&self, seq: u64) {
        let _ = self.events.send(Rebuilding::Finished { seq });
    }
}

/// A case's hold on the worker, made before the loop runs and moved to the client's thread.
struct RebuildHold {
    gate: Arc<RebuildGate>,
    events: crossbeam_channel::Receiver<Rebuilding>,
}

/// A case that ends lets every rebuild through, so the loop's drain on close finishes.
impl Drop for RebuildHold {
    fn drop(&mut self) {
        self.open();
    }
}

/// How long a case waits for the worker to start or finish a rebuild.
const REBUILD_WAIT: std::time::Duration = std::time::Duration::from_secs(120);

impl RebuildHold {
    /// Let every rebuild through, from now on.
    fn open(&self) {
        *self.gate.permits.lock().unwrap() = None;
        self.gate.turn.notify_all();
    }

    /// Let no further rebuild through until a release or an open.
    fn close(&self) {
        *self.gate.permits.lock().unwrap() = Some(0);
    }

    /// Let `count` more rebuilds through.
    fn release(&self, count: usize) {
        if let Some(left) = self.gate.permits.lock().unwrap().as_mut() {
            *left += count;
        }
        self.gate.turn.notify_all();
    }

    /// The next rebuild to start: its edit and the documents it reads.
    fn started(&self) -> (u64, BTreeMap<Url, String>) {
        loop {
            match self.events.recv_timeout(REBUILD_WAIT) {
                Ok(Rebuilding::Started { seq, docs }) => return (seq, docs),
                Ok(Rebuilding::Finished { .. }) => {}
                Err(err) => panic!("no rebuild started: {err}"),
            }
        }
    }

    /// The edit the next rebuild to finish ends at.
    fn finished(&self) -> u64 {
        loop {
            match self.events.recv_timeout(REBUILD_WAIT) {
                Ok(Rebuilding::Finished { seq }) => return seq,
                Ok(Rebuilding::Started { .. }) => {}
                Err(err) => panic!("no rebuild finished: {err}"),
            }
        }
    }

    /// Every rebuild event not read yet.
    fn rest(&self) -> Vec<Rebuilding> {
        self.events.try_iter().collect()
    }
}

impl Server {
    /// Put the worker behind a gate, closed: no rebuild runs until the case releases one.
    fn hold_rebuilds(&mut self) -> RebuildHold {
        let (events, inbox) = crossbeam_channel::unbounded();
        let gate = Arc::new(RebuildGate {
            permits: std::sync::Mutex::new(Some(0)),
            turn: std::sync::Condvar::new(),
            events,
        });
        self.rebuilds.gate = Some(Arc::clone(&gate));
        RebuildHold {
            gate,
            events: inbox,
        }
    }

    /// Take `request` as the loop does, then wait for every rebuild it waits on.
    fn handle_request(&mut self, request: Request) -> Result<()> {
        self.request(request)?;
        self.drain().map(drop)
    }

    /// Take `notification` as the loop does, then install every rebuild it causes.
    fn handle_notification(&mut self, notification: Notification) -> Result<()> {
        let taken = self.notification(notification);
        self.drain()?;
        taken
    }

}
