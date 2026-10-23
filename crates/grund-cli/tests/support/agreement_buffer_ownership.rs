//! Real buffer constructors retain their cwd and bytes (§AR-ci.10.1, §AR-ci.10.2).

use super::*;
use crate::agreement_tree::fixture_checks::{legacy_candidate, lifetimes, occupied};
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

thread_local! {
    static SCHEDULE: RefCell<Option<(Sender<PathBuf>, Receiver<()>)>> = const { RefCell::new(None) };
}

/// Test-only scheduling seam; ordinary fixture calls have no schedule installed.
pub(super) fn before_git(root: &std::path::Path) {
    SCHEDULE.with(|slot| {
        if let Some((ready, resume)) = slot.borrow_mut().take() {
            ready.send(root.to_path_buf()).unwrap();
            resume.recv_timeout(Duration::from_secs(30)).unwrap();
        }
    });
}

#[test]
fn grund_977_buffer_lifetimes() {
    lifetimes(
        || Tree::new("977-buffer-lifetimes", &config(&[]), &[]),
        |tree| &tree.0,
    );
}

#[test]
fn grund_977_buffer_occupied_root() {
    let name = "977-buffer-occupied";
    occupied(
        legacy_candidate(name),
        || Tree::new(name, &config(&[]), &[]),
        |tree| &tree.0,
    );
}

#[test]
fn grund_977_buffer_create_spawn_interleave() {
    let (ready_tx, ready_rx) = channel();
    let (resume_tx, resume_rx) = channel();
    let paused = std::thread::spawn(move || {
        SCHEDULE.with(|slot| *slot.borrow_mut() = Some((ready_tx, resume_rx)));
        Tree::new("977-buffer-interleave", &config(&[]), &[])
    });
    let created = ready_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    fs::write(created.join("owner-sentinel"), b"paused").unwrap();
    let competitor = Tree::new("977-buffer-interleave", &config(&[]), &[]);
    let distinct = created.canonicalize().unwrap() != competitor.0;
    drop(competitor);
    let preserved = fs::read(created.join("owner-sentinel")).ok().as_deref() == Some(b"paused");
    resume_tx.send(()).unwrap();
    let result = paused.join();
    println!(
        "interleave distinct={distinct} sentinel_preserved={preserved} actual_git_init_ok={}",
        result.is_ok()
    );
    assert!(
        distinct && preserved && result.is_ok(),
        "competing fixture invalidated cwd before actual git init"
    );
}
