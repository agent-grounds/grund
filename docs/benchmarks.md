# Local benchmark report

This archival report preserves the 2026-05-20 local wall-clock run for the `grund`
repo and a historical instruction-count snapshot. The tables, samples and provenance
below are historical evidence, not measurements of the current release or a general
speed ranking against Lychee.

## Instruction-Count Baseline

The table below is the historical committed instruction-count snapshot from
`cargo bench -p grund --features bench --locked --bench instructions -- --save-summary=json`
on 2026-05-20. Its inputs include this repository as it then stood; it is not the
baseline that current pull-request CI uses.

| Benchmark | Input | Instructions (`Ir`) | Estimated Cycles |
|---|---|---:|---:|
| `check` | this repo | 299,672,739 | 431,943,109 |
| `check_large_10k` | generated 10k-file fixture | 1,055,099,244 | 1,475,491,249 |
| `list` | this repo | 289,680,670 | 417,771,050 |
| `show_brief` | this repo | 284,142,202 | 409,995,654 |
| `show` | this repo | 284,116,880 | 410,041,898 |
| `show_full` | this repo | 290,702,550 | 419,471,481 |
| `refs` | this repo | 284,135,903 | 410,102,297 |
| `cover` | this repo | 301,086,562 | 433,510,661 |
| `fmt_check` | this repo | 349,977,643 | 502,904,379 |

Current benchmarks use generated fixtures rather than the live repository
([§AR-benchmarks.1.1](architecture/AR-benchmarks.md#11-generated-fixtures-never-this-repository)).
Pull-request CI measures the base branch and then the PR commit, recording the `Ir`
comparison in summaries; pushes to `main` record counts without comparing against
themselves ([§AR-ci.5.1](architecture/AR-ci.md#51-pull-requests-and-pushes)).
Instruction-regression limits are not enforced: growth is reported but does not fail
the build ([§AR-ci.5.2](architecture/AR-ci.md#52-regression-limits)).

Callgrind instruction count is a repeatable workload-cost proxy for a given binary
and input, not elapsed time or a universal latency guarantee
([§AR-benchmarks.2](architecture/AR-benchmarks.md#2-why-instruction-counts-not-wall-clock)).
Comparisons must control the input, invocation, toolchain and build settings. Different
binaries, including PGO versus non-PGO builds, can change counts independently of
the source change under review. Compare revisions under consistent build conditions
([§AR-benchmarks.5](architecture/AR-benchmarks.md#5-comparing-two-revisions)); PGO
is excluded from development CI ([§AR-ci.6](architecture/AR-ci.md#6-pgo-stays-out-of-development-ci)).
The archival provenance below does not record compiler flags or PGO state; it cannot
establish comparability with a differently built binary today.

## Instructions

Keep this report as an archive. Write future local measurements to a separate file
from the repository root, retaining their own date, machine, binary and workload context:

```sh
mkdir -p "$HOME/ag/tmp"
python3 scripts/local-benchmark-report.py --out "$HOME/ag/tmp/grund-local-benchmark-report.md"
```

Useful options:

- `--warm-runs N` changes the number of warm samples after the cold run.
- `--grund PATH` points at a specific `grund` binary; by default the script uses `target/release/grund` when present.
- `--lychee PATH` points at a specific Lychee binary.
- `--lychee-path PATH` may be repeated to replace the default Lychee inputs: `README.md docs examples`.

To measure a locally built release binary, build it before running the command above.
This build choice alone does not make the two tools' workloads equivalent:

```sh
cargo build --release --locked
```

## Method

- Cold time is the first measured invocation of each exact command in this script run. The script does not use `sudo` and does not drop the OS page cache.
- Warm time is the median of 7 immediate subsequent invocations with command output suppressed.
- Timings use Python `time.perf_counter()` around the whole subprocess, so process startup and argument parsing are included.
- Lychee may perform URL work and may benefit from its own cache or network conditions; `grund` works only over the local scanned tree.

## Machine

| Field | Value |
|---|---|
| Timestamp | 2026-05-20T03:51:55+02:00 |
| Host | kung-fu-workstation |
| Os | Linux-6.17.0-29-generic-x86_64-with-glibc2.39 |
| Kernel | Linux kung-fu-workstation 6.17.0-29-generic #29~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Mon May 11 10:30:58 UTC 2 x86_64 x86_64 |
| Cpu | Intel(R) Core(TM) Ultra 9 285K |
| Logical Cpus | 24 |
| Memory | 93.7 GiB |
| Repo Disk Free | 13.2 GiB |
| Python | 3.11.5 |

## Tool Versions

| Tool | Version |
|---|---|
| `grund` | `grund 0.4.0` |
| `lychee` | `lychee 0.23.0` |
| Git commit | `2cd13fff1ee00a7decde6d8768531fdb3eb3a894` |
| Git branch | `rm-parallel-scan` |
| Working tree dirty | `no` |

## Workload

| Tool | Local work checked |
|---|---:|
| `grund check .` | 335 declarations + 2,184 citations across 98 scanned files (21,458 lines) |
| `lychee --include-fragments README.md docs examples` | 1,083 links across 80 markup files |

## Markdown Link Token Impact

Markdown cross-reference links are generated presentation over the underlying citation text ([§FS-fmt.6](functional-spec/FS-fmt.md#6-cross-reference-emission)). To quantify that prompt cost, this snapshot measured temporary copies under `/tmp/grund-link-impact` with `tiktoken` `o200k_base`. The workload was the 70 Markdown files in this repo's configured `grund` scan scope; non-Markdown scanned source files and the root `README.md` were not counted.

| Form | Tokens |
|---|---:|
| Bare `§...` citations | 150,287 |
| Markdown links `[§...](...)` | 180,444 |
| Delta | +30,157 tokens |

## Results

| Command | Cold | Warm median | Warm min | Warm max |
|---|---:|---:|---:|---:|
| `target/release/grund check .` | 0.032s | 0.030s | 0.029s | 0.034s |
| `target/release/grund fmt --check .` | 0.040s | 0.037s | 0.036s | 0.038s |
| `lychee --no-progress --include-fragments README.md docs examples` | 1.170s | 0.535s | 0.504s | 0.563s |

## Throughput

At the warm median in this 2026-05-20 run, `grund check .` scans about 722k lines
of source per second. This is local throughput for the recorded binary and scanned
tree, not a current-release performance guarantee.

## Comparison

The commands above perform different workloads: `grund` scans local declarations
and citations; the configured Lychee run checks links, including URL work affected
by caches and network conditions. The originally reported warm-median elapsed-time
ratio was 18.0 for these runs. Citation edges and links are different units, and that ratio
does not establish general relative speed or an interchangeable cost per checked item.

## Raw Warm Samples

- `grund check`: 0.031s, 0.032s, 0.034s, 0.030s, 0.029s, 0.029s, 0.029s
- `grund fmt --check`: 0.037s, 0.038s, 0.038s, 0.036s, 0.037s, 0.036s, 0.036s
- `lychee`: 0.559s, 0.504s, 0.525s, 0.535s, 0.563s, 0.529s, 0.559s
