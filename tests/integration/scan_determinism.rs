//! §REQ-deterministic-output — same input, same bytes, whatever the thread
//! count. The scan is rayon-parallel and merges per-file results afterwards
//! (§AR-scanner), and the e2e determinism pass runs each tiny fixture twice
//! under one pool, so the merge order was never exercised. Every
//! report-producing command over this repository, and `check` over every
//! plain-check fixture, must be byte-identical — stdout, stderr and exit — at
//! one thread and at eight.

#[path = "binaries.rs"]
mod binaries;
#[path = "corpus.rs"]
mod corpus;

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const THREADS: &[&str] = &["1", "8"];
const REPO_COMMANDS: &[&[&str]] = &[
    &["check", "--full"],
    &["check", "--full", "--format", "json"],
    &["check", "--full", "--suggestions"],
    &["cover", "--format=json"],
    &["cover"],
    &["list"],
    &["list", "--format", "json"],
    &["list", "--unused"],
];
const MIN_CASES: usize = 80;

#[derive(PartialEq, Eq, Debug)]
struct Run {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    code: Option<i32>,
}

fn run(grund: &Path, cwd: &Path, args: &[&str], threads: &str) -> Run {
    let output = Command::new(grund)
        .args(args)
        .current_dir(cwd)
        .env("RAYON_NUM_THREADS", threads)
        .output()
        .unwrap_or_else(|err| panic!("run grund {args:?}: {err}"));
    Run {
        stdout: output.stdout,
        stderr: output.stderr,
        code: output.status.code(),
    }
}

fn differences(grund: &Path, cwd: &Path, args: &[&str]) -> Option<String> {
    let baseline = run(grund, cwd, args, THREADS[0]);
    for threads in &THREADS[1..] {
        let other = run(grund, cwd, args, threads);
        if other != baseline {
            return Some(format!(
                "grund {} in {}: RAYON_NUM_THREADS={} and ={threads} differ\n  exit {:?} vs {:?}\n  stdout {} vs {} bytes\n  stderr {} vs {} bytes",
                args.join(" "),
                cwd.display(),
                THREADS[0],
                baseline.code,
                other.code,
                baseline.stdout.len(),
                other.stdout.len(),
                baseline.stderr.len(),
                other.stderr.len()
            ));
        }
    }
    Some(baseline)
        .filter(|run| !matches!(run.code, Some(0) | Some(1)))
        .map(|run| {
            format!(
                "grund {} in {} exited {:?} — the invocation itself is broken, so it compares nothing:\n{}",
                args.join(" "),
                cwd.display(),
                run.code,
                String::from_utf8_lossy(&run.stderr)
            )
        })
}

fn workers() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// `work` over every item on every CPU libtest may use, results in item order
/// (§REQ-one-answer.proof.delivery.payment): a fixture's one-thread run and its
/// eight-thread rerun stay one pair on one worker, only different fixtures overlap, and
/// every comparison and message is the serial sweep's (§REQ-deterministic-output.1).
fn in_order<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let mut results: Vec<Option<R>> = items.iter().map(|_| None).collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers().clamp(1, items.len().max(1)))
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(index) else {
                            return done;
                        };
                        done.push((index, work(item)));
                    }
                })
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(done) => {
                    for (index, result) in done {
                        results[index] = Some(result);
                    }
                }
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
    });
    results
        .into_iter()
        .map(|result| result.expect("every item ran"))
        .collect()
}

#[test]
fn every_report_is_byte_identical_at_one_thread_and_at_eight() {
    let started = Instant::now();
    let repo = binaries::repo_root();
    let grund = binaries::grund();
    let mut problems: Vec<String> =
        in_order(REPO_COMMANDS, |args| differences(&grund, &repo, args))
            .into_iter()
            .flatten()
            .collect();
    let selection = corpus::plain_check_cases(&repo);
    let per_case = in_order(&selection.cases, |case| {
        let baseline = run(
            &grund,
            &case.root,
            &["check", ".", "--format", "json"],
            THREADS[0],
        );
        let mut found = Vec::new();
        for threads in &THREADS[1..] {
            let other = run(
                &grund,
                &case.root,
                &["check", ".", "--format", "json"],
                threads,
            );
            if other != baseline {
                found.push(format!(
                    "{}: check differs at {} and {threads} threads",
                    case.name, THREADS[0]
                ));
            }
        }
        found
    });
    let compared = per_case.len();
    problems.extend(per_case.into_iter().flatten());
    eprintln!(
        "scan determinism: {} repository command(s) and {compared} fixture(s) compared at {} thread counts",
        REPO_COMMANDS.len(),
        THREADS.len()
    );
    // §AR-ci.requirements.5.1: past libtest's capture, the seconds this sweep still costs.
    let _ = writeln!(
        std::io::stderr(),
        "gate time: scan determinism {} repository command(s) and {compared} fixture(s) on {} worker(s) in {:.1}s",
        REPO_COMMANDS.len(),
        workers(),
        started.elapsed().as_secs_f64()
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(compared >= MIN_CASES, "only {compared} fixtures compared");
}
