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
//! §AR-benchmarks.1.2.2: `scripts/bench_shards.py` says which rows this run measures:
//! `GRUND_BENCH_SHARD=<i>/<n>` one shard of the `pr` tier, `GRUND_BENCH_TIER=nightly` every
//! row, and neither the whole `pr` tier.
//!
//! §AR-benchmarks.1.7: a row with a `frontend` measures `grund-lsp` fed its editor session
//! on stdin, or the Python extension counted from its entry point.
//!
//! §AR-benchmarks.5: the binary under test is `GRUND_BENCH_BIN` when it is set, so
//! one harness on one fixture bundle measures a base revision's binary and its own.
//!
//! Run with `cargo bench -p grund --features bench --bench instructions` (needs Valgrind
//! and `iai-callgrind-runner` on `PATH`, and `grund-lsp` built beside `grund`).

#[cfg(feature = "bench")]
use iai_callgrind::{
    Bench, BinaryBenchmark, BinaryBenchmarkConfig, BinaryBenchmarkGroup, Callgrind, Command,
    EventKind, Stdin, binary_benchmark_group, main,
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
/// §AR-benchmarks.1.2.2: one shard of the `pr` tier, `<i>/<n>`.
#[cfg(feature = "bench")]
const SHARD_VAR: &str = "GRUND_BENCH_SHARD";
/// §AR-benchmarks.1.2.1: `nightly` measures every row of both tiers.
#[cfg(feature = "bench")]
const TIER_VAR: &str = "GRUND_BENCH_TIER";
/// §AR-benchmarks.1.7: the `grund-lsp` binary a `frontend: lsp` row runs; beside `grund` when
/// unset, and under `GRUND_BENCH_BIN` the base revision has none unless it names one.
#[cfg(feature = "bench")]
const LSP_BIN_VAR: &str = "GRUND_BENCH_LSP_BIN";
/// §AR-benchmarks.1.7: the built Python extension a `frontend: python` row loads.
#[cfg(feature = "bench")]
const PYTHON_EXT_VAR: &str = "GRUND_BENCH_PYTHON_EXT";
/// §AR-benchmarks.1.7: the pinned interpreter a `frontend: python` row runs; `python3` when unset.
#[cfg(feature = "bench")]
const PYTHON_VAR: &str = "GRUND_BENCH_PYTHON";
/// §AR-benchmarks.1.7: the extension's entry point and the body of each engine worker thread,
/// where Callgrind's collection toggles on; a toggle holds for its own thread only.
#[cfg(feature = "bench")]
const PYTHON_ENTRIES: [&str; 2] = ["_native::call", "rayon_core::registry::ThreadBuilder::run"];
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
    /// §AR-benchmarks.1.2: the one feature the generator adds to this fixture.
    feature: Option<String>,
    /// §AR-benchmarks.1.2: a workspace of this many plain member projects.
    members: Option<usize>,
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
    /// §AR-benchmarks.1.2: lines fed on stdin, `{id}` the subject, as `show --batch` reads them.
    stdin: Option<Vec<String>>,
    /// §AR-benchmarks.1.2: a file under the fixture named instead of its root.
    path: Option<String>,
    /// §AR-benchmarks.1.7: `lsp` or `python`; absent for a row that runs `grund`.
    frontend: Option<String>,
    limit_pct: f64,
    ceiling_ir: u64,
}

#[cfg(feature = "bench")]
fn catalog() -> Catalog {
    serde_json::from_str(CATALOG).expect("benches/scenarios.json is a scenario catalog")
}

/// §AR-benchmarks.1.2.2: the names of the rows this run measures, as `bench_shards.py`
/// derives them from `GRUND_BENCH_SHARD` and `GRUND_BENCH_TIER`.
#[cfg(feature = "bench")]
fn selected() -> Vec<String> {
    let mut command = ProcessCommand::new("python3");
    command
        .arg(Path::new(REPO).join("scripts/bench_shards.py"))
        .arg("select");
    if let Some(tier) = std::env::var_os(TIER_VAR) {
        command.arg("--tier").arg(tier);
    }
    if let Some(shard) = std::env::var_os(SHARD_VAR) {
        command.arg(shard);
    }
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .expect("run scripts/bench_shards.py");
    assert!(
        output.status.success(),
        "scripts/bench_shards.py refused {SHARD_VAR} / {TIER_VAR}"
    );
    String::from_utf8(output.stdout)
        .expect("bench_shards.py prints row names")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// §AR-ci.5.2: the row's relative `soft_limits`, and its `hard_limits` ceiling only when the
/// ceilings are switched on and the binary is the harness's own. §AR-benchmarks.1.7: a Python
/// row counts from the extension's entry point, not from the interpreter's start.
#[cfg(feature = "bench")]
fn limits(scenario: &Scenario) -> BinaryBenchmarkConfig {
    let mut callgrind = Callgrind::default();
    if scenario.frontend.as_deref() == Some("python") {
        callgrind.args(
            std::iter::once("collect-atstart=no".to_owned())
                .chain(PYTHON_ENTRIES.map(|entry| format!("toggle-collect={entry}"))),
        );
    }
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

/// §AR-benchmarks.1.7: the program a row runs — `grund`, `grund-lsp` or the interpreter —
/// or why the binary under test has none.
#[cfg(feature = "bench")]
fn program(scenario: &Scenario) -> Result<PathBuf, String> {
    match scenario.frontend.as_deref() {
        None => Ok(grund()),
        Some("lsp") => match std::env::var_os(LSP_BIN_VAR) {
            Some(binary) => Ok(PathBuf::from(binary)),
            None if std::env::var_os(BENCH_BIN_VAR).is_some() => {
                Err(format!("no base `grund-lsp`: {LSP_BIN_VAR} is unset"))
            }
            None => Ok(beside_grund("grund-lsp")),
        },
        Some("python") => {
            if std::env::var_os(BENCH_BIN_VAR).is_some()
                && std::env::var_os(PYTHON_EXT_VAR).is_none()
            {
                return Err(format!(
                    "no base Python extension: {PYTHON_EXT_VAR} is unset"
                ));
            }
            Ok(
                std::env::var_os(PYTHON_VAR)
                    .map_or_else(|| PathBuf::from("python3"), PathBuf::from),
            )
        }
        Some(other) => panic!("`{}` names the unknown frontend `{other}`", scenario.name),
    }
}

/// A binary Cargo built beside `grund`, which a frontend row needs built first.
#[cfg(feature = "bench")]
fn beside_grund(name: &str) -> PathBuf {
    let path = Path::new(GRUND).with_file_name(name);
    // §AR-benchmarks.1.7: a frontend binary is built by the caller, never by the harness.
    assert!(
        path.is_file(),
        "{} is not built; build it with the profile `cargo bench` uses, or name it in the \
         environment",
        path.display()
    );
    path
}

/// The Python extension a `frontend: python` row loads.
#[cfg(feature = "bench")]
fn python_extension() -> OsString {
    std::env::var_os(PYTHON_EXT_VAR)
        .unwrap_or_else(|| beside_grund("lib_native.so").into_os_string())
}

/// A scenario's program, its arguments and its stdin, held together so the probe and
/// Callgrind run one invocation.
#[cfg(feature = "bench")]
struct Invocation {
    program: PathBuf,
    args: Vec<OsString>,
    stdin: Option<PathBuf>,
}

/// A scenario's invocation, or why its setup failed under the binary being measured.
#[cfg(feature = "bench")]
type Setup = Result<Invocation, String>;

/// Under the scenario's own binary a failed setup fails the harness loudly.
#[cfg(feature = "bench")]
fn command(setup: Setup) -> Command {
    let invocation = setup.unwrap_or_else(|reason| panic!("benchmark setup failed: {reason}"));
    let mut command = Command::new(invocation.program);
    command.args(invocation.args);
    if let Some(stdin) = invocation.stdin {
        command.stdin(Stdin::File(stdin));
    }
    command.build()
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
    if std::env::var_os(BENCH_BIN_VAR).is_none() {
        return true;
    }
    let invocation = match setup() {
        Ok(invocation) => invocation,
        Err(reason) => {
            refuse(scenario, &reason);
            return false;
        }
    };
    let mut probe = ProcessCommand::new("python3");
    probe
        .arg(Path::new(REPO).join("scripts/bench_record.py"))
        .arg(PROBE)
        .arg("--scenario")
        .arg(scenario)
        .arg("--bin")
        .arg(&invocation.program)
        .arg("--refused")
        .arg(refused_file());
    if let Some(stdin) = &invocation.stdin {
        probe.arg("--stdin").arg(stdin);
    }
    probe
        .arg("--")
        .args(&invocation.args)
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
    if let Some(feature) = &fixture.feature {
        command.arg("--feature").arg(feature);
    }
    if let Some(members) = fixture.members {
        command.arg("--members").arg(members.to_string());
    }
    let status = command.status().expect("run benchmark fixture generator");
    assert!(status.success(), "benchmark fixture generator failed");
    root
}

/// Write a row's stdin under `target/bench-stdin/`, outside every fixture and outside
/// Callgrind.
#[cfg(feature = "bench")]
fn stdin_file(scenario: &Scenario, extension: &str, bytes: &[u8]) -> PathBuf {
    let directory = Path::new(REPO).join("target/bench-stdin");
    std::fs::create_dir_all(&directory).expect("create target/bench-stdin");
    let path = directory.join(format!("{}.{extension}", scenario.name));
    std::fs::write(&path, bytes).expect("write a benchmark's stdin");
    path
}

/// §AR-benchmarks.1.7, §DA-benchmark-frontends: the editor session `train.py` defines for
/// PGO training, printed for this fixture, for `grund-lsp` to read on stdin.
#[cfg(feature = "bench")]
fn lsp_session(scenario: &Scenario, root: &Path) -> PathBuf {
    let output = ProcessCommand::new("python3")
        .arg(Path::new(REPO).join("scripts/distribution/train.py"))
        .arg("session")
        .arg("--root")
        .arg(root)
        .stderr(Stdio::inherit())
        .output()
        .expect("run scripts/distribution/train.py session");
    assert!(output.status.success(), "train.py session failed");
    stdin_file(scenario, "jsonrpc", &output.stdout)
}

/// §AR-benchmarks.1.2: a row's invocation — its `args`, `{id}` its fixture's subject, then
/// the fixture's path, or the file `path` names under it — after its setup has run under the
/// binary being measured, so the probe of §AR-benchmarks.5.1 runs exactly what Callgrind
/// measures. §AR-benchmarks.1.3 is the one setup today: `fmt --write` normalizes
/// `fmt_check`'s input, so the measured `--check` exits successfully on a canonical tree.
/// §AR-benchmarks.1.7: a frontend row's `args` name what its frontend runs instead.
#[cfg(feature = "bench")]
fn invocation(catalog: &Catalog, scenario: &Scenario) -> Setup {
    let fixture = &catalog.fixtures[&scenario.fixture];
    let program = program(scenario)?;
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
    let with_subject = |arg: &String| {
        if arg.contains(SUBJECT) {
            arg.replace(SUBJECT, subject())
        } else {
            arg.clone()
        }
    };
    let target = scenario
        .path
        .as_ref()
        .map_or_else(|| root.clone(), |path| root.join(path));
    match scenario.frontend.as_deref() {
        Some("lsp") => Ok(Invocation {
            program,
            args: Vec::new(),
            stdin: Some(lsp_session(scenario, &root)),
        }),
        Some("python") => {
            let mut args: Vec<OsString> = vec![
                "-I".into(),
                Path::new(REPO)
                    .join("scripts/distribution/train.py")
                    .into_os_string(),
                "python-call".into(),
                python_extension(),
                PathBuf::from(REPO).into_os_string(),
                Path::new(REPO)
                    .join("target/bench-python")
                    .join(&scenario.name)
                    .into_os_string(),
                target.into_os_string(),
            ];
            args.extend(scenario.args.iter().map(|arg| with_subject(arg).into()));
            Ok(Invocation {
                program,
                args,
                stdin: None,
            })
        }
        _ => {
            let mut args: Vec<OsString> = scenario
                .args
                .iter()
                .map(|arg| with_subject(arg).into())
                .collect();
            args.push(target.into_os_string());
            let stdin = scenario.stdin.as_ref().map(|lines| {
                let text: String = lines.iter().map(|line| with_subject(line) + "\n").collect();
                stdin_file(scenario, "ndjson", text.as_bytes())
            });
            Ok(Invocation {
                program,
                args,
                stdin,
            })
        }
    }
}

/// §AR-benchmarks.1.2, §AR-benchmarks.5.1: one benchmark per selected catalog row the
/// binary under test accepts, keyed by the row's bare name so the run record of
/// §AR-benchmarks.7.1 is.
#[cfg(feature = "bench")]
fn commands_group(group: &mut BinaryBenchmarkGroup) {
    if std::env::var_os(BENCH_BIN_VAR).is_some() {
        // A refusal is this run's alone, never a previous run's.
        let _ = std::fs::remove_file(refused_file());
    }
    let catalog = catalog();
    let selected = selected();
    // §AR-benchmarks.1.3: a row's setup runs after every other row's fixture is ensured, so
    // no later regeneration undoes it before Callgrind measures.
    let (plain, set_up): (Vec<&Scenario>, Vec<&Scenario>) = catalog
        .scenarios
        .iter()
        .filter(|scenario| selected.contains(&scenario.name))
        .partition(|scenario| scenario.setup.is_none());
    for scenario in plain.into_iter().chain(set_up) {
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
