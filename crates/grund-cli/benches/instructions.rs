//! Instruction-counting benches for the hot CLI commands. Each runs a `grund`
//! binary under Callgrind (`iai-callgrind`) for a deterministic, CI-stable
//! instruction count.
//!
//! §AR-benchmarks: every benchmark scans a *generated fixture*, never this repo —
//! a fixed input isolates a genuine code slowdown (what the gate must fail on)
//! from the repo merely doing more work as its own docs/config evolve. See the
//! spec for the full rationale, the fixture set, and why one fixture enables
//! `[citations]` while the rest omit it.
//!
//! §AR-benchmarks.5: the binary under test is `GRUND_BENCH_BIN` when it is set, so
//! one harness on one fixture bundle measures a base revision's binary and its own.
//!
//! The command list also drives the PGO training run in `scripts/pgo-build.sh`;
//! keep them in sync. Run with
//! `cargo bench -p grund --features bench --bench instructions` (needs Valgrind and
//! `iai-callgrind-runner` on `PATH`).

#[cfg(feature = "bench")]
use iai_callgrind::{
    BinaryBenchmarkConfig, BinaryBenchmarkGroup, Callgrind, Command, EventKind, binary_benchmark,
    binary_benchmark_attribute, binary_benchmark_group, main,
};
#[cfg(feature = "bench")]
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
};

/// The freshly built `grund` binary (Cargo exports this env var).
#[cfg(feature = "bench")]
const GRUND: &str = env!("CARGO_BIN_EXE_grund");
/// §AR-benchmarks.5: names another `grund` binary to measure under this harness.
#[cfg(feature = "bench")]
const BENCH_BIN_VAR: &str = "GRUND_BENCH_BIN";
/// §AR-benchmarks.5.1: the file the probe lists the scenarios `GRUND_BENCH_BIN` refuses in;
/// `target/bench-refused.txt` when unset.
#[cfg(feature = "bench")]
const REFUSED_VAR: &str = "GRUND_BENCH_REFUSED";
/// The `scripts/bench_record.py` mode that probes one scenario.
#[cfg(feature = "bench")]
const PROBE: &str = "probe";
/// Repository root — used only to locate the fixture generator script, never
/// scanned by a benchmark.
#[cfg(feature = "bench")]
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
/// Generated canonical fixture for the per-command suite — large enough to be
/// representative, small enough to keep the suite quick.
#[cfg(feature = "bench")]
const CANONICAL_REPO_REL: &str = "target/bench-fixtures/canonical-repo";
#[cfg(feature = "bench")]
const CANONICAL_FILE_COUNT: usize = 1_500;
/// Generated large conformant fixture for the `grund check` budget benchmark.
#[cfg(feature = "bench")]
const LARGE_REPO_REL: &str = "target/bench-fixtures/large-conformant-repo";
#[cfg(feature = "bench")]
const LARGE_REPO_FILE_COUNT: usize = 10_000;

/// A declared ID the canonical fixture always contains (`id_for(1)` in the
/// generator) — the subject of the `show` ladder and `refs` benchmarks. The
/// fixture's bodies are uniform, so these measure the scan + render path that
/// dominates, not body size.
#[cfg(feature = "bench")]
const FIXTURE_ID: &str = "FS-00001-feature-00001";

/// Generated canonical fixture that declares `[citations]`, so `check_citations`
/// exercises the citing-side classification + direction passes (§FS-config.3.9).
#[cfg(feature = "bench")]
const CITATIONS_REPO_REL: &str = "target/bench-fixtures/canonical-citations-repo";

/// §AR-ci.5.2: the `Ir` growth over the base-branch baseline past which the bench
/// fails. A benchmark's own config replaces the group's regression config, so
/// `check_large_10k` holds the tighter limit and every other bench the group's.
#[cfg(feature = "bench")]
const GROUP_IR_LIMIT_PCT: f64 = 12.5;
#[cfg(feature = "bench")]
const LARGE_IR_LIMIT_PCT: f64 = 5.0;

#[cfg(feature = "bench")]
fn ir_limit(pct: f64) -> BinaryBenchmarkConfig {
    let mut config = BinaryBenchmarkConfig::default();
    config.tool(Callgrind::default().soft_limits([(EventKind::Ir, pct)]));
    config
}

/// §AR-benchmarks.5: the binary under test — `GRUND_BENCH_BIN`, else the freshly built one.
#[cfg(feature = "bench")]
fn grund() -> PathBuf {
    std::env::var_os(BENCH_BIN_VAR).map_or_else(|| PathBuf::from(GRUND), PathBuf::from)
}

/// A scenario's arguments, held apart from the binary so the probe and Callgrind run one list.
#[cfg(feature = "bench")]
#[derive(Default)]
struct Invocation(Vec<OsString>);

#[cfg(feature = "bench")]
impl Invocation {
    fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.0.push(arg.into());
        self
    }

    fn args<const N: usize>(self, args: [&str; N]) -> Self {
        args.into_iter().fold(self, Self::arg)
    }
}

#[cfg(feature = "bench")]
fn command(invocation: Invocation) -> Command {
    Command::new(grund()).args(invocation.0).build()
}

#[cfg(feature = "bench")]
fn refused_file() -> PathBuf {
    std::env::var_os(REFUSED_VAR).map_or_else(
        || Path::new(REPO).join("target/bench-refused.txt"),
        PathBuf::from,
    )
}

/// §AR-benchmarks.5.1: under `GRUND_BENCH_BIN`, run a scenario's invocation once outside
/// Callgrind; a scenario the base binary refuses is listed and left out of the base run.
#[cfg(feature = "bench")]
fn accepts(scenario: &str, invocation: fn() -> Invocation) -> bool {
    let Some(binary) = std::env::var_os(BENCH_BIN_VAR) else {
        return true;
    };
    ProcessCommand::new("python3")
        .arg(Path::new(REPO).join("scripts/bench_record.py"))
        .arg(PROBE)
        .arg("--scenario")
        .arg(scenario)
        .arg("--bin")
        .arg(binary)
        .arg("--refused")
        .arg(refused_file())
        .arg("--")
        .args(invocation().0)
        .status()
        .expect("run the benchmark probe")
        .success()
}

#[cfg(feature = "bench")]
fn ensure_fixture(rel: &str, file_count: usize, citations: bool) -> PathBuf {
    let root = Path::new(REPO).join(rel);
    let script = Path::new(REPO).join("scripts/generate_large_benchmark_fixture.py");
    let mut command = ProcessCommand::new("python3");
    command
        .arg(script)
        .arg("--root")
        .arg(&root)
        .arg("--files")
        .arg(file_count.to_string());
    if citations {
        command.arg("--citations");
    }
    let status = command.status().expect("run benchmark fixture generator");
    assert!(status.success(), "benchmark fixture generator failed");
    root
}

#[cfg(feature = "bench")]
fn canonical_repo() -> PathBuf {
    ensure_fixture(CANONICAL_REPO_REL, CANONICAL_FILE_COUNT, false)
}

/// Generate and normalize the formatter benchmark input outside Callgrind so
/// the measured `--check` exits successfully on a canonical tree, under the binary
/// being measured. §AR-benchmarks.1.3
#[cfg(feature = "bench")]
fn canonical_fmt_repo() -> PathBuf {
    let root = canonical_repo();
    let status = ProcessCommand::new(grund())
        .args(["fmt", "--write"])
        .arg(&root)
        .stdout(Stdio::null())
        .status()
        .expect("normalize formatter benchmark fixture");
    assert!(status.success(), "formatter benchmark setup failed");
    root
}

// Each scenario's invocation is a function of its own, so the probe of §AR-benchmarks.5.1
// runs exactly what Callgrind measures.

// `grund check <fixture>` — validate every citation in the tree.
#[cfg(feature = "bench")]
fn check_args() -> Invocation {
    Invocation::default().args(["check"]).arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn check() -> Command {
    command(check_args())
}

// `grund check <large-fixture>` — the 10k-file budget input.
#[cfg(feature = "bench")]
fn check_large_10k_args() -> Invocation {
    Invocation::default().args(["check"]).arg(ensure_fixture(
        LARGE_REPO_REL,
        LARGE_REPO_FILE_COUNT,
        false,
    ))
}

#[cfg(feature = "bench")]
#[binary_benchmark(config = ir_limit(LARGE_IR_LIMIT_PCT))]
fn check_large_10k() -> Command {
    command(check_large_10k_args())
}

// `grund check <fixture-with-citations>` — the citation-direction code path
// (classify + obligation + prohibition passes, §FS-config.3.9). Same file count
// as `check`, so the delta against it is the direction-checking overhead.
#[cfg(feature = "bench")]
fn check_citations_args() -> Invocation {
    Invocation::default().args(["check"]).arg(ensure_fixture(
        CITATIONS_REPO_REL,
        CANONICAL_FILE_COUNT,
        true,
    ))
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn check_citations() -> Command {
    command(check_citations_args())
}

// `grund list <fixture>` — every declared ID.
#[cfg(feature = "bench")]
fn list_args() -> Invocation {
    Invocation::default().args(["list"]).arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn list() -> Command {
    command(list_args())
}

// `grund <ID> --brief <fixture>` — title plus first paragraph.
#[cfg(feature = "bench")]
fn show_brief_args() -> Invocation {
    Invocation::default()
        .args(["show", FIXTURE_ID, "--brief"])
        .arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn show_brief() -> Command {
    command(show_brief_args())
}

// `grund <ID> <fixture>` — the lead-default declaration read.
#[cfg(feature = "bench")]
fn show_args() -> Invocation {
    Invocation::default()
        .args(["show", FIXTURE_ID])
        .arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn show() -> Command {
    command(show_args())
}

// `grund <ID> --full <fixture>` — one full declaration body.
#[cfg(feature = "bench")]
fn show_full_args() -> Invocation {
    Invocation::default()
        .args(["show", FIXTURE_ID, "--full"])
        .arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn show_full() -> Command {
    command(show_full_args())
}

// `grund refs <ID> <fixture>` — every citation site of an ID.
#[cfg(feature = "bench")]
fn refs_args() -> Invocation {
    Invocation::default()
        .args(["refs", FIXTURE_ID])
        .arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn refs() -> Command {
    command(refs_args())
}

// `grund cover <fixture>` — the citation graph grouped by scanned file.
#[cfg(feature = "bench")]
fn cover_args() -> Invocation {
    Invocation::default().args(["cover"]).arg(canonical_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn cover() -> Command {
    command(cover_args())
}

// `grund fmt --check <fixture>` — report (without writing) any non-canonical citation.
#[cfg(feature = "bench")]
fn fmt_check_args() -> Invocation {
    Invocation::default()
        .args(["fmt", "--check"])
        .arg(canonical_fmt_repo())
}

#[cfg(feature = "bench")]
#[binary_benchmark]
fn fmt_check() -> Command {
    command(fmt_check_args())
}

/// Add each scenario the binary under test accepts (§AR-benchmarks.5.1).
#[cfg(feature = "bench")]
macro_rules! scenarios {
    ($group:ident: $($bench:ident => $args:ident),+ $(,)?) => {
        $(
            if accepts(stringify!($bench), $args) {
                $group.binary_benchmark(binary_benchmark_attribute!($bench));
            }
        )+
    };
}

#[cfg(feature = "bench")]
fn commands_group(group: &mut BinaryBenchmarkGroup) {
    if std::env::var_os(BENCH_BIN_VAR).is_some() {
        // A refusal is this run's alone, never a previous run's.
        let _ = std::fs::remove_file(refused_file());
    }
    scenarios!(group:
        check => check_args,
        check_large_10k => check_large_10k_args,
        check_citations => check_citations_args,
        list => list_args,
        show_brief => show_brief_args,
        show => show_args,
        show_full => show_full_args,
        refs => refs_args,
        cover => cover_args,
        fmt_check => fmt_check_args,
    );
}

#[cfg(feature = "bench")]
binary_benchmark_group!(
    name = commands;
    config = ir_limit(GROUP_IR_LIMIT_PCT);
    benchmarks = |group: &mut BinaryBenchmarkGroup| commands_group(group)
);

#[cfg(feature = "bench")]
main!(binary_benchmark_groups = commands);

#[cfg(not(feature = "bench"))]
fn main() {}
