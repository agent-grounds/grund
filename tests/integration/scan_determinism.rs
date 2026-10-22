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
#[path = "ordered_pool.rs"]
mod ordered_pool;

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use ordered_pool::{in_order, workers};

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
    // A fixture's one-thread run and its eight-thread rerun stay one pair on one worker.
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
