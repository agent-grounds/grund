# Embed grund from Python

Build the local extension from a checkout with CPython 3.10+ and Rust:

```sh
python -m pip install .
```

The Python API is supplied locally; PyPI publication and release wheels remain
pending ([§FS-distribution.3.3.7](../functional-spec/FS-distribution.md#337-local-source-and-typing-handoff)).
This runnable example uses the repository's existing erroneous-citation fixture:

```python
from grund import check, show

repo = "tests/e2e/cases/json-report/repo"
result = check(repo)
for finding in result.report:
    print(finding.code, finding.line)  # dangling 3
assert "FS-999-missing" in show("FS-001-alpha", root=repo, mode="brief").body
```

The API prints nothing itself. A completed check returns `CheckResult` even
when its report has findings. `report` is the complete report; `selected_report`
applies selectors. Both contain errors, warnings and suggestions tuples.
Iteration yields those groups in order. Findings retain severity/channel, code,
nullable path/line/column, message, every site, and rule authority. Suggestions
have channel `suggestion` and severity None. `run_cautions` carries separate
run warnings; it is never folded into the report
([§FS-distribution.3.3.1](../functional-spec/FS-distribution.md#331-typed-immutable-results)).

## Functions and defaults

Required operands are positional. Other options are keywords. Every disk-tree
operation below also accepts `root=None`, except `scan(root)` (required) and
`init(target=None)`. `check` accepts root positionally. `integrations` is
user-global and setup instructions have no root. The callable definitions and
inline annotations are in `python/grund`; `py.typed` marks them for type checkers.
The full inventory implements
[§FS-distribution.3.3.5](../functional-spec/FS-distribution.md#335-complete-initial-inventory).

| Function (besides common root) | Returned record |
| --- | --- |
| `check(root=None, *, require_grounding=False, suggestions=False, full=False, rule=None, only=(), ignore=(), only_rule=False)` | `CheckResult`: reports, scan-error status, output format, cautions |
| `scan(root)` | `ScanResult`: catalog, citations, scanned files, scan errors, cautions |
| `show(id, *, section=None, mode="lead", format="text")` | `ShowResult`: body, logical path, line, nullable rendered JSON, sections, cautions |
| `show_batch(queries=None, *, mode="lead")` | `BatchResult`: ordered records with query, nullable result/failure, cautions |
| `refs(id, *, section=None, descendants=False)` | `RefsResult`: hits, scan errors, note, kind title, file summaries, site/file totals, cautions |
| `list_ids(*, kinds=(), projects=(), unused=False, selector=None)` | `ListResult`: entries, kind summaries, scan errors, cautions |
| `list_sizes(*, kinds=(), projects=(), unused=False, selector=None, units=("lines","words","bytes"), top=None)` | `SizesResult`: ordered lead/full measurements, scan errors, cautions |
| `cover(*, text=False, lines=())` | `CoverResult` or `CoverTextResult`: file-grouped coverage, scan errors, cautions; with non-empty `lines`, `CoverLinesResult`: one ownership record per range, scan errors, cautions |
| `fmt(*, write=False, marker=False, cross_refs=False)` | `FmtResult`: changes, scan errors, refused writes, cautions |
| `propose_id(kind, title, *, width=3)` | `IdProposal`: ID, kind, nullable number/home hints, slug, cautions |
| `init(target=None, *, name=None, description=None, docs=False, force=False, write=False, check=False, no_vcs=False, agents=None)` | `InitResult`: events, errors, notes, nullable next guidance, pending-change status, cautions |
| `effective_config()` / `validate_config()` | `ConfigResult`: effective schema mapping, config warnings, cautions |
| `fetch(id, *, write=False)` | `FetchResult`: completion and cautions; requires opt-in |
| `integrations(client=None, *, write=False, conversation=None, conversation_target=None, agent=None)` | `IntegrationsResult`: detection, client descriptors, nullable artifact, events, manual steps, cautions |
| `complete_ids(prefix="", *, sections=False)` | `CompletionResult`: candidates, cautions |
| `reference_style()` | `ReferenceStyle`: marker, typing trigger, cautions |
| `agent_setup_instructions()` | `SetupInstructions`: canonical text, cautions |

Results are frozen dataclasses, collections are tuples, and optional fields are
present as None. Effective config, failure details and partial opaque payloads
are recursively read-only mappings. `ShowQuery(id, section=None)` is frozen.
Batch queries accept strings or ShowQuery records; None discovers every
coordinate, while an empty sequence succeeds without loading config.
Batch success records contain the complete ShowResult, with JSON supplied by
the engine's batch read; each unsuccessful query retains a Failure in its record.

`cover(lines=...)` takes strings, each `"N"` or `"N-M"` exactly as `grund cover
--lines` takes it, so a refusal names the range as written; `root` names the one
file asked about, and `text` must stay False. Each record carries the range and
its owner runs, each owner run its section runs, as
[§FS-cover.6.3](../functional-spec/FS-cover.md#63-output) describes.

`mode` accepts lead/brief/toc/full; `format` accepts text/md/json. Filters and
selectors take string sequences. Size units are lines/words/bytes, top is
positive and width is non-negative. Ignore wins over only; `only_rule` requires
rule. Selectors never suppress safety `io` findings. All resolution, scope,
scan exclusions and kind overrides are engine decisions.

Shell scripts, stdin/NDJSON, process/watch lifecycle, exit codes and LSP
transport remain process frontend concerns. Overlay/on-type/hover utilities
are outside this initial disk-backed API.

## Failures, paths and concurrency

`GrundError` has `ConfigError`, `FilesystemError`, `QueryError` and fallback
`OperationError` subclasses. `.failure` retains kind, code/message, nullable
location, sites, authority, causes, cautions, partial output and details such as
candidates or OS error codes. Single unsuccessful queries raise QueryError;
batch query failures continue in records, while a batch setup failure raises
once. Unknown ID kinds raise OperationError and rejected ID proposals raise
QueryError. Wrong Python types raise TypeError; invalid options raise ValueError
([§FS-distribution.3.3.2](../functional-spec/FS-distribution.md#332-operational-failures-and-invalid-arguments)).

Paths accept str or os.PathLike[str]. Omitted root snapshots cwd; explicit files
and directories retain explicit engine scope. Bytes raise TypeError, surrogate
paths raise PathEncodingError (a ValueError), and NUL raises ValueError before
work. Logical result paths use `/`. The core preflight also refuses the known
unsafe unnamed non-Unicode workspace alias before deriving it; setting an
explicit project_name can make that member representable. Calls do not change
cwd, read process argv or exit the interpreter
([§FS-distribution.3.3.3](../functional-spec/FS-distribution.md#333-per-call-scope-and-path-encoding)).

Calls are synchronous and release the GIL during Rust work. Independent reads
may overlap. Serialize writers to the same files. Pending interrupts are checked
on return: the operation may finish before KeyboardInterrupt arrives. No async
API, mid-call cancellation, extra rollback or arbitrary panic recovery is
promised. CPython with the GIL is required; PyPy and free-threaded builds are
unsupported ([§FS-distribution.3.3.4](../functional-spec/FS-distribution.md#334-silent-synchronous-calls)).

## Writes and integrations

`fmt()` previews. `fmt(write=True)` applies managed rewrites; config-enabled
cross-reference formatting still applies when cross_refs=False. `init()`
previews, `init(write=True)` scaffolds, and `init(check=True)` suppresses writes
even if write=True. Force never replaces existing config. Fetch has no preview:
`fetch(id, write=True)` explicitly authorizes materialization; omission refuses
before loading config or executing a fetcher. Reads execute no fetcher
([§FS-distribution.3.3.6](../functional-spec/FS-distribution.md#336-explicit-mutation-opt-ins)).

`integrations()` inspects detected clients. A client operand returns its
artifact. Writes use existing managed ownership and agent-in-use gates, retain
manual instructions, and preserve manual text. Clients, preferences and agents
use the engine's current vocabularies; invalid combinations refuse before
writing. A scoped agent requires a conversation_target. Conversation options
require write=True; a clientless write requires a conversation option.

The [runnable example](../../examples/python-api/) and
[native source handoff](../../crates/grund-py/README.md) accompany this guide.
The parity corpus targets Rust/Python; Node joins through #469. Local compilation
alone does not establish passing parity or clean-source install evidence.
