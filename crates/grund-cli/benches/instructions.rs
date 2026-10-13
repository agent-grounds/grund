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
//! §AR-benchmarks.1.2: the scenarios are the rows of `benches/scenarios.json`, and this
//! harness builds its one group from them; adding a row adds a benchmark. The catalog's
//! `pgo` subset is also the PGO training list `scripts/distribution/train.py` reads
//! (§AR-benchmarks.1.5).
//!
//! §AR-benchmarks.5: the binary under test is `GRUND_BENCH_BIN` when it is set, so
//! one harness on one fixture bundle measures a base revision's binary and its own.
//!
//! Run with `cargo bench -p grund --features bench --bench instructions` (needs Valgrind
//! and `iai-callgrind-runner` on `PATH`).

#[cfg(feature = "bench")]
use iai_callgrind::{
    Bench, BinaryBenchmark, BinaryBenchmarkConfig, BinaryBenchmarkGroup, Callgrind, Command,
    EventKind, binary_benchmark_group, main,
};
#[cfg(feature = "bench")]
use serde::Deserialize;
#[cfg(feature = "bench")]
use std::{
    collections::BTreeMap,
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
/// §AR-ci.5.2: switches the absolute `Ir` ceilings on, which CI's series alone is held to.
#[cfg(feature = "bench")]
const CEILINGS_VAR: &str = "GRUND_BENCH_CEILINGS";
/// The `scripts/bench_record.py` mode that probes one scenario.
#[cfg(feature = "bench")]
const PROBE: &str = "probe";
/// §AR-benchmarks.1.2: the scenario catalog, compiled in so a changed row rebuilds the harness.
#[cfg(feature = "bench")]
const CATALOG: &str = include_str!("scenarios.json");
/// The placeholder in a row's `args` that stands for its fixture's `subject`.
#[cfg(feature = "bench")]
const SUBJECT: &str = "{id}";

/// Repository root — used only to locate the fixture generator script, never
/// scanned by a benchmark.
#[cfg(feature = "bench")]
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// §AR-benchmarks.1.2: the catalog — the generated fixtures and the ordered scenario rows.
/// A row's `ceiling_from` and `pgo` are for its readers, not the harness.
#[cfg(feature = "bench")]
#[derive(Deserialize)]
struct Catalog {
    fixtures: BTreeMap<String, Fixture>,
    scenarios: Vec<Scenario>,
}

/// §AR-benchmarks.1.1: one generated fixture, and the declared ID its single-ID rows read.
#[cfg(feature = "bench")]
#[derive(Deserialize)]
struct Fixture {
    dir: String,
    files: usize,
    citations: bool,
    subject: Option<String>,
}

/// One benchmark: its invocation on its fixture, and its two limits (§AR-ci.5.2).
#[cfg(feature = "bench")]
#[derive(Deserialize)]
struct Scenario {
    name: String,
    args: Vec<String>,
    fixture: String,
    /// §AR-benchmarks.1.3: run once outside Callgrind, on the same fixture, before measuring.
    setup: Option<Vec<String>>,
    limit_pct: f64,
    ceiling_ir: u64,
}

#[cfg(feature = "bench")]
fn catalog() -> Catalog {
    serde_json::from_str(CATALOG).expect("benches/scenarios.json is a scenario catalog")
}

/// §AR-ci.5.2: the row's relative `soft_limits`, and its `hard_limits` ceiling only when the
/// ceilings are switched on and the binary is the harness's own.
#[cfg(feature = "bench")]
fn limits(scenario: &Scenario) -> BinaryBenchmarkConfig {
    let mut callgrind = Callgrind::default();
    callgrind.soft_limits([(EventKind::Ir, scenario.limit_pct)]);
    if std::env::var_os(CEILINGS_VAR).is_some() && std::env::var_os(BENCH_BIN_VAR).is_none() {
        callgrind.hard_limits([(EventKind::Ir, scenario.ceiling_ir)]);
    }
    let mut config = BinaryBenchmarkConfig::default();
    config.tool(callgrind);
    config
}

/// §AR-benchmarks.5: the binary under test — `GRUND_BENCH_BIN`, else the freshly built one.
#[cfg(feature = "bench")]
fn grund() -> PathBuf {
    std::env::var_os(BENCH_BIN_VAR).map_or_else(|| PathBuf::from(GRUND), PathBuf::from)
}

/// A scenario's arguments, held apart from the binary so the probe and Callgrind run one list.
#[cfg(feature = "bench")]
type Invocation = Vec<OsString>;

/// A scenario's invocation, or why its setup failed under the binary being measured.
#[cfg(feature = "bench")]
type Setup = Result<Invocation, String>;

/// Under the scenario's own binary a failed setup fails the harness loudly.
#[cfg(feature = "bench")]
fn command(setup: Setup) -> Command {
    let invocation = setup.unwrap_or_else(|reason| panic!("benchmark setup failed: {reason}"));
    Command::new(grund()).args(invocation).build()
}

#[cfg(feature = "bench")]
fn refused_file() -> PathBuf {
    std::env::var_os(REFUSED_VAR).map_or_else(
        || Path::new(REPO).join("target/bench-refused.txt"),
        PathBuf::from,
    )
}

/// §AR-benchmarks.5.1: under `GRUND_BENCH_BIN`, run a scenario's setup and invocation once
/// outside Callgrind; a scenario the base binary refuses, in either, is listed and left out
/// of the base run.
#[cfg(feature = "bench")]
fn accepts(scenario: &str, setup: impl FnOnce() -> Setup) -> bool {
    let Some(binary) = std::env::var_os(BENCH_BIN_VAR) else {
        return true;
    };
    let invocation = match setup() {
        Ok(invocation) => invocation,
        Err(reason) => {
            refuse(scenario, &reason);
            return false;
        }
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
        .args(invocation)
        .status()
        .expect("run the benchmark probe")
        .success()
}

/// List a scenario whose setup the base binary refused, as the probe lists a refused run.
#[cfg(feature = "bench")]
fn refuse(scenario: &str, reason: &str) {
    use std::io::Write as _;
    let path = refused_file();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create the refused-scenario list's directory");
    }
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("open the refused-scenario list");
    writeln!(out, "{scenario}").expect("list the refused scenario");
    eprintln!(
        "the base binary refuses the setup of `{scenario}` ({reason}); it is no base, not compared"
    );
}

/// §AR-benchmarks.1.1: generate the fixture outside Callgrind; the generator leaves an
/// unchanged one as it is.
#[cfg(feature = "bench")]
fn ensure_fixture(fixture: &Fixture) -> PathBuf {
    let root = Path::new(REPO).join(&fixture.dir);
    let script = Path::new(REPO).join("scripts/generate_large_benchmark_fixture.py");
    let mut command = ProcessCommand::new("python3");
    command
        .arg(script)
        .arg("--root")
        .arg(&root)
        .arg("--files")
        .arg(fixture.files.to_string());
    if fixture.citations {
        command.arg("--citations");
    }
    let status = command.status().expect("run benchmark fixture generator");
    assert!(status.success(), "benchmark fixture generator failed");
    root
}

/// §AR-benchmarks.1.2: a row's invocation — its `args`, `{id}` its fixture's subject, then
/// the fixture's path — after its setup has run under the binary being measured, so the
/// probe of §AR-benchmarks.5.1 runs exactly what Callgrind measures. §AR-benchmarks.1.3
/// is the one setup today: `fmt --write` normalizes `fmt_check`'s input, so the measured
/// `--check` exits successfully on a canonical tree.
#[cfg(feature = "bench")]
fn invocation(catalog: &Catalog, scenario: &Scenario) -> Setup {
    let fixture = &catalog.fixtures[&scenario.fixture];
    let root = ensure_fixture(fixture);
    if let Some(setup) = &scenario.setup {
        let status = ProcessCommand::new(grund())
            .args(setup)
            .arg(&root)
            .stdout(Stdio::null())
            .status()
            .expect("run the benchmark's setup");
        if !status.success() {
            return Err(format!("`{}` exited with {status}", setup.join(" ")));
        }
    }
    let subject = || {
        fixture.subject.as_deref().unwrap_or_else(|| {
            panic!(
                "`{}` reads `{SUBJECT}`, but its fixture names no subject",
                scenario.name
            )
        })
    };
    let mut invocation: Invocation = scenario
        .args
        .iter()
        .map(|arg| {
            if arg.contains(SUBJECT) {
                OsString::from(arg.replace(SUBJECT, subject()))
            } else {
                OsString::from(arg)
            }
        })
        .collect();
    invocation.push(root.into_os_string());
    Ok(invocation)
}

/// §AR-benchmarks.1.2, §AR-benchmarks.5.1: one benchmark per catalog row the binary under
/// test accepts, keyed by the row's bare name so the run record of §AR-benchmarks.7.1 is.
#[cfg(feature = "bench")]
fn commands_group(group: &mut BinaryBenchmarkGroup) {
    if std::env::var_os(BENCH_BIN_VAR).is_some() {
        // A refusal is this run's alone, never a previous run's.
        let _ = std::fs::remove_file(refused_file());
    }
    let catalog = catalog();
    for scenario in &catalog.scenarios {
        if accepts(&scenario.name, || invocation(&catalog, scenario)) {
            let name = scenario.name.as_str();
            group.binary_benchmark(
                BinaryBenchmark::new(name)
                    .config(limits(scenario))
                    .bench(Bench::new(name).command(command(invocation(&catalog, scenario)))),
            );
        }
    }
}

#[cfg(feature = "bench")]
binary_benchmark_group!(
    name = commands;
    benchmarks = |group: &mut BinaryBenchmarkGroup| commands_group(group)
);

#[cfg(feature = "bench")]
main!(binary_benchmark_groups = commands);

#[cfg(not(feature = "bench"))]
fn main() {}
