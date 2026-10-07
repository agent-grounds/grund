# Embed grund in Node

The Node frontend exposes the shared Rust engine through Promises, with typed
readonly records ([§FS-distribution.3.2.1](../functional-spec/FS-distribution.md#321-operations-options-and-refusals),
[§FS-distribution.3.2.2](../functional-spec/FS-distribution.md#322-typed-readonly-host-records)).
This is a locally buildable API-only rehearsal of the planned `grund-cli`
package. Publishing and executable/platform package assembly are separate work.
Use Node 22.x or 24.x.

## Build a local package

Install the Rust 1.95.0 toolchain, Node/npm and a linker/SDK for your host. Cargo
needs dependency access or populated caches. In a checkout:

```sh
node crates/grund-node/stage.mjs \
  --out-dir "$HOME/ag/tmp/grund-node-package" \
  --target-dir "$HOME/ag/tmp/grund-node-target"
cd "$HOME/ag/tmp/grund-node-package"
npm run build:source -- --target-dir "$HOME/ag/tmp/grund-node-target"
npm pack
```

Install the resulting tarball into a consumer with
`npm install --ignore-scripts /absolute/path/to/grund-cli-0.16.2-dev.tgz`.
Rebuild an unpacked installed package explicitly with `npm run build:source`.
That uses its bundled, locked source closure; it does not require checkout
access. Import/install never compiles or downloads a native addon
([§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership)).

```js
import { check, show } from 'grund-cli';
const report = await check('./repo', { suggestions: true });
const body = await show('FS-check', { root: './repo', mode: 'brief' });
console.log(report.report.errors, body.body);
```

CommonJS is `const { check, show } = require('grund-cli')`. Both modules use
the same `GrundError` class and loader. The declarations select the matching
ESM/CJS types under NodeNext.

## Operations

All calls return Promises. Options are plain objects; unknown keys, malformed
Unicode, NUL, wrong types/enums and conflicting options reject as input errors.
Omission selects the documented default; explicit null is supported only on
nullable fields. Every tree call captures cwd before dispatch; roots may be
relative to that captured cwd. Output paths and byte columns follow core rules.

| Call | Options and returned data |
| --- | --- |
| `check(root?, options?)` | requireGrounding, suggestions, full, rule, only, ignore, onlyRule; complete report, selectedReport, outputFormat, hadScanErrors |
| `scan(root)` | required explicit root; owned snapshot arrays, including declarations, citations, structures and value facts |
| `show(id, options?)` | root, section, mode=lead/brief/toc/full, format=text/md/json; body, coordinates, sections, exact core JSON and manifest |
| `showBatch(queries, options?)` | root, mode; null discovers all coordinates, [] returns before discovery; ordered success/failure records |
| `refs(id, options?)` | root, section, descendants; hits, summaries, totals, note and scanErrors |
| `list(options?)` | root, kinds, projects, unused, selector; entries and summaries |
| `listSizes(options?)` | list options, units (lines/words/bytes), positive top; ordered nullable measurements |
| `cover(options?)` | root; files including empty files and full citing-side coordinates |
| `fmt(options?)` | root, write=false, marker=false, crossRefs=false; changes, scanErrors, refusedWrites |
| `proposeId(kind, title, options?)` | root, width=3; proposed ID and authoring hints, no writes |
| `init(target?, options?)` | name, description, docs, force, dryRun, check, noVcs, agents; events, errors, notes, next, pendingChanges |
| `completeIds(options?)` | root, prefix, sections; sorted deduplicated ids |
| `effectiveConfig(options?)` | root; root/configFile provenance and effective schema-keyed values |
| `validateConfig(options?)` | root; same record, validating workspace/member configs without declaration scans |
| `fetch(id, options?)` | root; explicit configured fetch execution, requested id returned |
| `integrations(options?)` | client, write, conversation, conversationTarget, agent; detection/artifact/install discriminated records |
| `referenceStyle(path)` | required document path; member-specific marker and trigger |
| `agentSetupInstructions()` | canonical setup instructions returned as data |

Agents for init are canonical/claude/gemini/pi/copilot/cursor/windsurf/zed;
null means automatic selection and an empty array is invalid. Integration
clients are codium/iterm2/kitty/tmux/vscode/wezterm. Conversation preferences
require write, and an agent-specific target also requires conversationTarget.
Core validates active-agent and user-config policy before writing.

## Results and refusals

Every result has `runCautions`, separate from check report warnings. Check
findings resolve normally, including partial file-read checks where
`hadScanErrors` is true. Strict scan failure rejects with available snapshot
partial. Batch query failures continue in order; setup failure rejects the batch.

```js
import { show, GrundError } from 'grund-cli';
try {
  await show('FS-missing', { root: './repo' });
} catch (error) {
  if (!(error instanceof GrundError)) throw error;
  const { kind, code, sites, partial, runCautions } = error.failure;
  console.log(kind, code, sites, partial, runCautions);
}
```

Kinds are input/query/config/io/operation/load/worker/native/busy. Failures
retain nullable path/line/column, multi-site diagnostics, authority, causes,
details and available partial output. Branch on kind and documented codes,
not localized messages. Values/configuration keys remain schema spelling;
source value numbers remain exact strings. Numeric overflow rejects rather
than truncating ([§FS-distribution.3.2.2.1](../functional-spec/FS-distribution.md#3221-common-records)).

## Workers and writers

Native work runs off the event loop; independent read calls overlap. Default
fmt previews, init writes unless dryRun/check, and only explicit fetch runs
fetchers. Core still governs owned bytes, whole-set preflight, home/VCS
guards, protected configuration and integration writes. Concurrent writers
across roots reject busy before side effects and release their lease on
return/error/unwind. No cancellation, rollback or external-writer snapshot
is promised ([§FS-distribution.3.2.3.1](../functional-spec/FS-distribution.md#3231-execution)).

## Handoff and evidence

See [the native frontend README](../../crates/grund-node/README.md) for builder,
loader metadata, shared-oracle provenance and verification recipes.
[The captured consumer example](../../examples/node-api/example.mjs) reads the
json-report fixture through a locally installed package. Runtime, TS, package,
silence, lifecycle and parity evidence comes from the acceptance suite, not
from compilation. #471 owns prebuilt platform certification and the combined
API/CLI assembly manifest. #470 owns the common oracle/comparator; Node extends
its same-source framing without substituting CLI output for full native data.

The initial API is pre-1.0; existing compatibility rules govern removals and
renames. “Adapt Node to #466” is a later internal adaptation preserving approved
host/wire behavior. #459/#463 proposals are not implemented here
([§FS-distribution.3.2.4](../functional-spec/FS-distribution.md#324-acceptance-evidence)).
