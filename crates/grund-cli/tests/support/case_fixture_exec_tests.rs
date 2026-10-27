//! §FS-examples.5.6.3: a fixture script the runner copies is never busy because a
//! sibling invocation's spawn inherited the copy's write descriptor.
use super::{copy_dir, run_grund};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const SPAWNERS: usize = 8;
/// §FS-examples.5.6.3.1: a constant, never a reading of the clock. A runner that let a
/// sibling inherit the copy's write descriptor went busy within the first five
/// attempts, so 100 keeps a twentyfold margin; the macOS runner manages about 120
/// attempts in 20 s beside eight spawners, so 100 finishes there within those 20 s.
const ATTEMPTS: usize = 100;
const ETXTBSY: i32 = 26;

/// One worker copies a case's executable fixture with the runner's own `copy_dir`
/// and starts it at once, as `grund fetch` starts `scripts/fetch-ticket`; the
/// others keep spawning through the runner's own `run_grund`, as sibling cases
/// do. The padding keeps the copy's write descriptor open long enough that a
/// sibling fork lands inside it on most attempts while the runner lets it. Every
/// attempt runs to completion and only a busy start fails the test: no time budget
/// bounds the loop, because how many attempts fit in a time measures the machine
/// rather than the overlap (§FS-examples.5.6.3.1).
#[test]
fn a_copied_fixture_script_is_never_busy_while_sibling_cases_spawn() {
    let scratch = std::env::temp_dir().join(format!("grund-fixture-exec-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    let fixture = scratch.join("fixture");
    let script = fixture.join("scripts/fetch-ticket");
    fs::create_dir_all(script.parent().unwrap()).unwrap();
    let mut body = String::from("#!/bin/sh\nexit 0\n");
    body.push_str(&"# padding that keeps the copy open\n".repeat(1 << 17));
    fs::write(&script, body).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let sibling = scratch.join("sibling");
    fs::create_dir_all(&sibling).unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let spawners = (0..SPAWNERS)
        .map(|_| {
            let (stop, sibling) = (Arc::clone(&stop), sibling.clone());
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    run_grund(&sibling, &sibling, &["--version".into()], "sibling");
                }
            })
        })
        .collect::<Vec<_>>();

    let mut busy = None;
    let mut attempts = 0;
    while attempts < ATTEMPTS {
        let copy = scratch.join(format!("copy-{attempts}"));
        copy_dir(&fixture, &copy);
        match Command::new(copy.join("scripts/fetch-ticket")).status() {
            Err(err) if err.raw_os_error() == Some(ETXTBSY) => busy = Some((attempts, err)),
            Err(err) => panic!("start the copied fixture script: {err}"),
            Ok(status) => assert!(status.success(), "fixture script exited {status}"),
        }
        let _ = fs::remove_dir_all(&copy);
        attempts += 1;
        if busy.is_some() {
            break;
        }
    }
    stop.store(true, Ordering::Relaxed);
    for spawner in spawners {
        spawner.join().expect("sibling spawner");
    }
    let _ = fs::remove_dir_all(&scratch);
    if let Some((attempt, err)) = busy {
        panic!(
            "attempt {} of {attempts}: the copied fixture script could not start: {err}; a \
             sibling spawn inherited copy_dir's write descriptor (§FS-examples.5.6.3)",
            attempt + 1
        );
    }
}
