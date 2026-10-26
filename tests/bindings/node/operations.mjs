// §FS-distribution.3.2.1 — every declared operation returns owned typed data.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import * as api from 'grund-cli';
const root = process.argv[2];
const require = createRequire(import.meta.url);
const cjs = require('grund-cli');
assert.equal(cjs.GrundError, api.GrundError);
const inventory = [
  'check', 'scan', 'show', 'showBatch', 'refs', 'list', 'listSizes', 'cover',
  'fmt', 'proposeId', 'init', 'completeIds', 'effectiveConfig', 'validateConfig',
  'fetch', 'integrations', 'referenceStyle', 'agentSetupInstructions',
];
for (const name of inventory) assert.equal(typeof api[name], 'function', name);
assert.deepEqual(Object.keys(api).sort(), [...inventory, 'GrundError'].sort(),
                 'no process/editor/placeholder API exports');
async function invoke(name, args) {
  const pending = api[name](...args);
  assert.ok(pending instanceof Promise, name);
  const result = await pending;
  assert.ok(Array.isArray(result.runCautions), name);
  return result;
}
const check = await invoke('check', [root]);
assert.deepEqual(await cjs.check(root), check);
assert.ok(check.report.errors.some(f => f.code === 'dangling'));
assert.deepEqual(check.report, check.selectedReport);
assert.equal(check.hadScanErrors, false);
for (const finding of [...check.report.errors, ...check.report.warnings,
                       ...check.report.suggestions]) {
  assert.ok(Object.hasOwn(finding, 'column'));
  assert.ok(Array.isArray(finding.sites));
  assert.ok(Array.isArray(finding.authority));
}
const selected = await invoke('check', [root, { only: ['dangling'], ignore: ['dangling'] }]);
assert.ok(selected.report.errors.some(f => f.code === 'dangling'));
assert.equal(selected.selectedReport.errors.length, 0);
// §FS-check.1.4.1, §FS-check.2.2.1.4: the selected view says what judged nothing.
const caseRepo = name => process.env.GRUND_TEST_REPO + '/tests/e2e/cases/' + name + '/repo';
const off = await invoke('check', [caseRepo('check-only-opt-in-off'), { only: ['oversized-lead'] }]);
const offLines = off.selectedReport.warnings.filter(f => f.code === 'selected-check-off');
assert.equal(offLines.length, 1);
assert.deepEqual([offLines[0].path, offLines[0].line], [null, null]);
assert.ok(!off.report.warnings.some(f => f.code === 'selected-check-off'));
const trial = await invoke('check', [caseRepo('check-rule-applies-to-nothing-only-rule'),
  { rule: 'Each e2e must cite at least one FS.', onlyRule: true }]);
assert.equal(trial.selectedReport.warnings.length, 1);
const [empty] = trial.selectedReport.warnings;
assert.equal(empty.code, 'empty-citation-obligation');
assert.equal(empty.message, '--rule applies to nothing — tests/e2e/ declares no e2e ID');
assert.deepEqual(empty.authority, ['--rule']);
const id = 'FS-001-alpha';
// §FS-distribution.3.2.2.2: brief and Markdown retain this fixture's heading.
const lead = 'References FS-002-beta and FS-999-missing.\n';
const headed = '# FS-001-alpha: Alpha\n\n' + lead;
for (const mode of ['lead', 'brief', 'toc', 'full']) {
  for (const format of ['text', 'md', 'json']) {
    const shown = await invoke('show', [id, { root, mode, format }]);
    assert.equal(shown.id, id);
    assert.equal(shown.section, null);
    assert.equal(typeof shown.body, 'string');
    assert.equal(shown.body, mode === 'brief' || format === 'md' ? headed : lead);
    assert.ok(shown.path.endsWith('FS-001-alpha.md'));
    assert.equal(shown.line, 1);
    assert.ok(Array.isArray(shown.sections));
    assert.equal(format === 'json', typeof shown.json === 'string');
  }
}
const scan = await invoke('scan', [root]);
for (const name of ['declarations', 'citations', 'escapedCitations', 'scannedFiles',
                   'walkedDirs', 'fileStructures', 'valueBindings',
                   'invalidValueDeclarations', 'invalidValueBindings',
                   'sectionHeadingsOutsideDeclarations', 'unmarkedHeadings', 'nearMissHeadings'])
  assert.ok(Array.isArray(scan.snapshot[name]), name);
assert.ok(scan.snapshot.declarations.some(d => d.id === id));
assert.ok(scan.snapshot.citations.some(c => c.id === 'FS-999-missing'));
const batch = await invoke('showBatch', [[id, 'FS-999-missing', id], { root }]);
assert.deepEqual(batch.records.map(r => r.ok), [true, false, true]);
assert.equal(batch.records[1].failure.kind, 'query');
assert.equal(batch.records[1].failure.code, 'not-found');
assert.deepEqual((await invoke('showBatch', [[], { root: '/absent-config' }])).records, []);
const exhaustive = await invoke('showBatch', [null, { root }]);
assert.ok(exhaustive.records.length >= 2);
const refs = await invoke('refs', ['FS-002-beta', { root }]);
assert.ok(refs.hits.length > 0);
assert.equal(refs.totals.sites, refs.hits.length);
assert.ok(refs.summaries.length > 0);
const undeclared = await invoke('refs', ['FS-999-missing', { root }]);
assert.ok(undeclared.hits.length > 0);
// §FS-distribution.3.2.2.2: only an uncited undeclared target needs a refs note.
assert.equal(undeclared.note, null);
const uncited = await invoke('refs', ['FS-998-unused', { root }]);
assert.deepEqual(uncited.hits, []);
// §FS-refs.2.1: an explicit root scopes the note and its project-list advice.
assert.equal(uncited.note,
  'FS-998-unused is neither declared nor cited in scan path `' + root +
  '` — run `grund list` to see project declarations, including outside this scan');
const list = await invoke('list', [{ root }]);
assert.ok(list.entries.some(e => e.id === id));
assert.ok(Array.isArray(list.summaries));
const sizes = await invoke('listSizes', [{ root, units: ['bytes', 'lines', 'words'], top: 1 }]);
assert.equal(sizes.entries.length, 1);
assert.deepEqual(sizes.entries[0].measurements.map(m => m.unit), ['bytes', 'lines', 'words']);
const cover = await invoke('cover', [{ root }]);
assert.ok(cover.entries.some(e => e.citations.length > 0));
const preview = await invoke('fmt', [{ root }]);
assert.ok(Array.isArray(preview.changes));
assert.ok(Array.isArray(preview.refusedWrites));
// §FS-distribution.3.2.2.2: each change carries its per-kind counts and its lock-file flag.
assert.ok(preview.changes.every(c => typeof c.counts.linksWritten === 'number'
  && Object.hasOwn(c, 'lockFile')));
const proposed = await invoke('proposeId', ['FS', 'Fresh snow', { root }]);
assert.equal(typeof proposed.id, 'string');
assert.equal(proposed.slug, 'fresh-snow');
assert.deepEqual((await invoke('completeIds', [{ root, prefix: 'FS-001' }])).ids, [id]);
for (const name of ['effectiveConfig', 'validateConfig']) {
  const config = await invoke(name, [{ root }]);
  assert.equal(typeof config.root, 'string');
  assert.ok(Object.hasOwn(config, 'configFile'));
  assert.ok(Object.hasOwn(config.values, 'reference'));
  assert.ok(!Object.hasOwn(config.values, 'projectName'));
}
assert.deepEqual(Object.keys(await invoke('referenceStyle', [root + '/docs/functional-spec/FS-001-alpha.md']))
  .sort(), ['marker', 'runCautions', 'trigger']);
// §FS-distribution.3.2.1: the whole guide `--full` prints, the core through the last section.
const setup = (await invoke('agentSetupInstructions', [])).instructions;
assert.ok(setup.includes('## Fresh repository') && setup.includes('<!-- END chapter-rules -->'));
// Init, explicit fetch and install successes use copied fixtures in mutations/fetch.mjs.
const integrations = await invoke('integrations', []);
assert.equal(integrations.mode, 'detection');
assert.ok(Array.isArray(integrations.clients));
assert.ok(Array.isArray(integrations.detected));
assert.ok((await invoke('init', [root, { dryRun: true, noVcs: true }])).events);
await assert.rejects(invoke('fetch', [id, { root }]), error => error instanceof api.GrundError);
